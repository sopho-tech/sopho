use crate::common::constants::EnvironmentVariable;
use crate::common::errors::ConfigurationError;
use crate::logging::constants::{DEFAULT_LOGS_FORMAT, LogsFormat};
use crate::logging::flat_json_format::FlatJsonFormat;
use std::env::VarError;
use tracing::Subscriber;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub fn logs_format_from_variable(value: Option<String>) -> Result<LogsFormat, ConfigurationError> {
    let Some(value) = value else {
        return Ok(DEFAULT_LOGS_FORMAT);
    };
    value
        .parse::<LogsFormat>()
        .map_err(|_| ConfigurationError::InvalidLogsFormat { value })
}

fn logs_format_from_environment() -> Result<LogsFormat, ConfigurationError> {
    let variable = EnvironmentVariable::SophoFeatureLogsFormat;
    match std::env::var(variable.as_ref()) {
        Ok(value) => logs_format_from_variable(Some(value)),
        Err(VarError::NotPresent) => logs_format_from_variable(None),
        Err(source) => Err(ConfigurationError::InvalidEnvironmentVariable { variable, source }),
    }
}

pub fn log_layer<LogSubscriber, LogWriter>(
    logs_format: LogsFormat,
    log_writer: LogWriter,
) -> Box<dyn Layer<LogSubscriber> + Send + Sync>
where
    LogSubscriber: Subscriber + for<'span> LookupSpan<'span>,
    LogWriter: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    match logs_format {
        LogsFormat::Json => tracing_subscriber::fmt::layer()
            .fmt_fields(JsonFields::new())
            .event_format(FlatJsonFormat)
            .with_writer(log_writer)
            .boxed(),
        LogsFormat::Text => tracing_subscriber::fmt::layer()
            .with_writer(log_writer)
            .boxed(),
    }
}

pub fn initialize_logging() -> Result<(), ConfigurationError> {
    dotenvy::dotenv().ok();
    let logs_format = logs_format_from_environment()?;
    let log_filter = EnvFilter::try_from_env(EnvironmentVariable::RustLog)
        .unwrap_or_else(|_| format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into());
    tracing_subscriber::registry()
        .with(log_filter)
        .with(log_layer(logs_format, std::io::stdout))
        .init();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{log_layer, logs_format_from_variable};
    use crate::common::errors::ConfigurationError;
    use crate::logging::constants::LogsFormat;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn the_logs_format_is_text_unless_set() {
        assert_eq!(logs_format_from_variable(None).unwrap(), LogsFormat::Text);
        assert_eq!(
            logs_format_from_variable(Some("text".to_string())).unwrap(),
            LogsFormat::Text
        );
        assert_eq!(
            logs_format_from_variable(Some("json".to_string())).unwrap(),
            LogsFormat::Json
        );
    }

    #[derive(Clone, Default)]
    struct CapturedLogLines {
        written_bytes: Arc<Mutex<Vec<u8>>>,
    }

    impl std::io::Write for CapturedLogLines {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.written_bytes.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'writer> MakeWriter<'writer> for CapturedLogLines {
        type Writer = CapturedLogLines;

        fn make_writer(&'writer self) -> Self::Writer {
            self.clone()
        }
    }

    fn json_lines_logged_by(log_something: impl FnOnce()) -> Vec<serde_json::Value> {
        let captured_log_lines = CapturedLogLines::default();
        let subscriber = tracing_subscriber::registry()
            .with(log_layer(LogsFormat::Json, captured_log_lines.clone()));
        tracing::subscriber::with_default(subscriber, log_something);
        let written_text =
            String::from_utf8(captured_log_lines.written_bytes.lock().unwrap().clone()).unwrap();
        written_text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn a_json_line_carries_the_ids_at_the_top_level_even_inside_a_nested_span() {
        let json_lines = json_lines_logged_by(|| {
            let request_span = tracing::error_span!(
                "request",
                request_id = "request-1",
                method = "GET",
                path = "/api/v1/tenant",
                user_id = tracing::field::Empty,
                tenant_id = tracing::field::Empty,
            );
            let _request_guard = request_span.enter();
            tracing::info!("before the person is known");
            request_span.record("user_id", "user-1");
            request_span.record("tenant_id", "tenant-1");
            let library_span = tracing::info_span!("chat", prompt = "what a person asked");
            let _library_guard = library_span.enter();
            tracing::warn!(
                attempt_count = 3,
                is_final_attempt = true,
                "inside a library span"
            );
        });

        assert_eq!(json_lines.len(), 2);
        let first_line = &json_lines[0];
        assert_eq!(first_line["message"], "before the person is known");
        assert_eq!(first_line["level"], "INFO");
        assert_eq!(first_line["request_id"], "request-1");
        assert_eq!(first_line["method"], "GET");
        assert_eq!(first_line["path"], "/api/v1/tenant");
        assert_eq!(first_line.get("user_id"), None);
        assert_eq!(first_line.get("tenant_id"), None);
        assert!(first_line["timestamp"].is_string());
        assert!(first_line["target"].is_string());

        let nested_line = &json_lines[1];
        assert_eq!(nested_line["message"], "inside a library span");
        assert_eq!(nested_line["level"], "WARN");
        assert_eq!(nested_line["request_id"], "request-1");
        assert_eq!(nested_line["user_id"], "user-1");
        assert_eq!(nested_line["tenant_id"], "tenant-1");
        assert_eq!(nested_line["attempt_count"], 3);
        assert_eq!(nested_line["is_final_attempt"], true);
        for removed_field in ["span", "spans", "prompt"] {
            assert_eq!(nested_line.get(removed_field), None, "{removed_field}");
        }
    }

    #[test]
    fn a_background_job_line_carries_its_job_and_tenant() {
        let json_lines = json_lines_logged_by(|| {
            let job_span = tracing::error_span!(
                "background_job",
                job = "suggested_question_refresh",
                tenant_id = "tenant-1"
            );
            let _job_guard = job_span.enter();
            tracing::error!("refresh failed");
        });

        assert_eq!(json_lines.len(), 1);
        assert_eq!(json_lines[0]["job"], "suggested_question_refresh");
        assert_eq!(json_lines[0]["tenant_id"], "tenant-1");
        assert_eq!(json_lines[0].get("request_id"), None);
    }

    #[test]
    fn a_line_outside_any_span_has_no_ids() {
        let json_lines = json_lines_logged_by(|| tracing::info!("starting up"));

        assert_eq!(json_lines.len(), 1);
        assert_eq!(json_lines[0]["message"], "starting up");
        for trace_id_field in ["request_id", "user_id", "tenant_id", "job"] {
            assert_eq!(json_lines[0].get(trace_id_field), None, "{trace_id_field}");
        }
    }

    #[test]
    fn an_unknown_logs_format_is_refused_naming_the_variable() {
        for invalid_value in ["JSON", "Json", " json", "true", "", "yaml"] {
            let error = logs_format_from_variable(Some(invalid_value.to_string())).unwrap_err();
            assert!(
                matches!(error, ConfigurationError::InvalidLogsFormat { .. }),
                "{invalid_value:?} was not refused"
            );
            assert!(
                error.to_string().contains("SOPHO_FEATURE_LOGS_FORMAT"),
                "{error}"
            );
        }
    }
}
