use crate::logging::constants::{
    LEVEL_LOG_FIELD, TARGET_LOG_FIELD, TIMESTAMP_LOG_FIELD, TRACE_ID_LOG_FIELDS,
};
use chrono::{SecondsFormat, Utc};
use serde_json::{Map, Value};
use std::fmt;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

pub struct FlatJsonFormat;

struct EventFieldCollector<'line> {
    log_line: &'line mut Map<String, Value>,
}

impl Visit for EventFieldCollector<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.log_line
            .insert(field.name().to_string(), Value::from(format!("{value:?}")));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.log_line
            .insert(field.name().to_string(), Value::from(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.log_line
            .insert(field.name().to_string(), Value::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.log_line
            .insert(field.name().to_string(), Value::from(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.log_line
            .insert(field.name().to_string(), Value::from(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.log_line
            .insert(field.name().to_string(), Value::from(value));
    }
}

impl<LogSubscriber, SpanFieldFormat> FormatEvent<LogSubscriber, SpanFieldFormat> for FlatJsonFormat
where
    LogSubscriber: Subscriber + for<'span> LookupSpan<'span>,
    SpanFieldFormat: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        context: &FmtContext<'_, LogSubscriber, SpanFieldFormat>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut log_line = Map::new();
        log_line.insert(
            TIMESTAMP_LOG_FIELD.to_string(),
            Value::from(Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true)),
        );
        log_line.insert(
            LEVEL_LOG_FIELD.to_string(),
            Value::from(event.metadata().level().as_str()),
        );
        log_line.insert(
            TARGET_LOG_FIELD.to_string(),
            Value::from(event.metadata().target()),
        );
        event.record(&mut EventFieldCollector {
            log_line: &mut log_line,
        });
        if let Some(span_scope) = context.event_scope() {
            for span in span_scope.from_root() {
                let span_extensions = span.extensions();
                let Some(formatted_span_fields) =
                    span_extensions.get::<FormattedFields<SpanFieldFormat>>()
                else {
                    continue;
                };
                if formatted_span_fields.is_empty() {
                    continue;
                }
                let Value::Object(span_fields) =
                    serde_json::from_str::<Value>(formatted_span_fields).map_err(|_| fmt::Error)?
                else {
                    return Err(fmt::Error);
                };
                for trace_id_field in TRACE_ID_LOG_FIELDS {
                    if let Some(trace_id_value) = span_fields.get(trace_id_field) {
                        log_line.insert(trace_id_field.to_string(), trace_id_value.clone());
                    }
                }
            }
        }
        writeln!(writer, "{}", Value::Object(log_line))
    }
}
