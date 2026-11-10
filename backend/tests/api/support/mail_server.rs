use crate::support::constants::{
    EMAIL_SENDER, MAIL_SERVER_API_PORT, MAIL_SERVER_CONTAINER_LABEL, MAIL_SERVER_CONTAINER_NAME,
    MAIL_SERVER_IMAGE_NAME, MAIL_SERVER_IMAGE_TAG, MAIL_SERVER_MESSAGE_LIMIT,
    MAIL_SERVER_READY_POLL_MILLISECONDS, MAIL_SERVER_READY_TIMEOUT_SECONDS,
    MAIL_SERVER_REFUSE_EVERY_RECIPIENT, MAIL_SERVER_SMTP_PORT, MAIL_SERVER_STARTUP_LOCK_FILE_NAME,
    MAIL_SERVER_TIMEOUT_SECONDS, REFUSING_MAIL_SERVER_CONTAINER_NAME,
    SILENT_MAIL_SERVER_TIMEOUT_SECONDS, SILENT_MAIL_SERVER_WAIT_SECONDS,
};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::StatusCode;
use serde_json::Value;
use sopho::{EmailConfiguration, SmtpTls};
use std::fs::File;
use std::net::TcpListener;
use std::time::{Duration, Instant};
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt, ReuseDirective};
use tokio::sync::OnceCell;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

pub struct MailServer {
    host: String,
    smtp_port: u16,
    api_base_url: String,
}

pub struct ReceivedEmail {
    pub subject: String,
    pub text: String,
    pub html: String,
}

static SHARED_MAIL_SERVER: OnceCell<MailServer> = OnceCell::const_new();
static SHARED_REFUSING_MAIL_SERVER: OnceCell<MailServer> = OnceCell::const_new();

pub async fn shared_mail_server() -> &'static MailServer {
    SHARED_MAIL_SERVER
        .get_or_init(|| start_or_reuse_mail_server(MAIL_SERVER_CONTAINER_NAME, None))
        .await
}

pub async fn shared_refusing_mail_server() -> &'static MailServer {
    SHARED_REFUSING_MAIL_SERVER
        .get_or_init(|| {
            start_or_reuse_mail_server(
                REFUSING_MAIL_SERVER_CONTAINER_NAME,
                Some(MAIL_SERVER_REFUSE_EVERY_RECIPIENT),
            )
        })
        .await
}

pub fn unreachable_email_configuration(public_url: &str) -> EmailConfiguration {
    let closed_port = TcpListener::bind("127.0.0.1:0")
        .expect("reserve a local port")
        .local_addr()
        .expect("read the reserved port")
        .port();
    email_configuration("127.0.0.1", closed_port, public_url)
}

pub struct SilentMailServer {
    pub smtp_port: u16,
    accepted_connections: UnboundedReceiver<()>,
}

impl SilentMailServer {
    pub async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind the silent mail server");
        let smtp_port = listener
            .local_addr()
            .expect("read the silent mail server port")
            .port();
        let (connection_sender, accepted_connections) = unbounded_channel();
        tokio::spawn(async move {
            let mut open_connections = Vec::new();
            loop {
                let (connection, _) = listener
                    .accept()
                    .await
                    .expect("accept a connection to the silent mail server");
                open_connections.push(connection);
                if connection_sender.send(()).is_err() {
                    break;
                }
            }
        });
        Self {
            smtp_port,
            accepted_connections,
        }
    }

    pub fn email_configuration(&self, public_url: &str) -> EmailConfiguration {
        EmailConfiguration {
            smtp_timeout_seconds: SILENT_MAIL_SERVER_TIMEOUT_SECONDS,
            ..email_configuration("127.0.0.1", self.smtp_port, public_url)
        }
    }

    pub async fn wait_for_connection(&mut self) {
        tokio::time::timeout(
            Duration::from_secs(SILENT_MAIL_SERVER_WAIT_SECONDS),
            self.accepted_connections.recv(),
        )
        .await
        .expect("nothing connected to the silent mail server in time")
        .expect("the silent mail server stopped");
    }
}

pub fn link_in(email: &ReceivedEmail, link_prefix: &str) -> String {
    email
        .text
        .split_whitespace()
        .find(|word| word.starts_with(link_prefix))
        .unwrap_or_else(|| panic!("no link starting with {link_prefix} in:\n{}", email.text))
        .to_string()
}

impl MailServer {
    pub fn email_configuration(&self, public_url: &str) -> EmailConfiguration {
        email_configuration(&self.host, self.smtp_port, public_url)
    }

    pub async fn emails_to(&self, recipient: &str) -> Vec<ReceivedEmail> {
        let client = reqwest::Client::new();
        let search: Value = client
            .get(format!(
                "{}/api/v1/search?query={}",
                self.api_base_url,
                utf8_percent_encode(&format!("to:\"{recipient}\""), NON_ALPHANUMERIC)
            ))
            .send()
            .await
            .expect("search the mail server")
            .json()
            .await
            .expect("read the mail server search");
        let mut received = Vec::new();
        for summary in search["messages"].as_array().expect("a list of messages") {
            let message_id = summary["ID"].as_str().expect("a message id");
            let message: Value = client
                .get(format!("{}/api/v1/message/{message_id}", self.api_base_url))
                .send()
                .await
                .expect("read a message from the mail server")
                .json()
                .await
                .expect("read the message body");
            received.push(ReceivedEmail {
                subject: message["Subject"].as_str().unwrap_or_default().to_string(),
                text: message["Text"].as_str().unwrap_or_default().to_string(),
                html: message["HTML"].as_str().unwrap_or_default().to_string(),
            });
        }
        received
    }
}

fn email_configuration(host: &str, smtp_port: u16, public_url: &str) -> EmailConfiguration {
    EmailConfiguration {
        smtp_host: host.to_string(),
        smtp_port,
        smtp_tls: SmtpTls::Unencrypted,
        smtp_credentials: None,
        smtp_timeout_seconds: MAIL_SERVER_TIMEOUT_SECONDS,
        from_address: EMAIL_SENDER.to_string(),
        public_url: public_url.to_string(),
    }
}

async fn start_or_reuse_mail_server(
    container_name: &'static str,
    allowed_recipients: Option<&'static str>,
) -> MailServer {
    let startup_lock = tokio::task::spawn_blocking(|| {
        let lock_file = File::create(std::env::temp_dir().join(MAIL_SERVER_STARTUP_LOCK_FILE_NAME))
            .expect("create the mail server startup lock file");
        lock_file
            .lock()
            .expect("lock the mail server startup lock file");
        lock_file
    })
    .await
    .expect("acquire the mail server startup lock");
    let image = GenericImage::new(MAIL_SERVER_IMAGE_NAME, MAIL_SERVER_IMAGE_TAG)
        .with_exposed_port(MAIL_SERVER_SMTP_PORT.tcp())
        .with_exposed_port(MAIL_SERVER_API_PORT.tcp())
        .with_env_var("MP_MAX_MESSAGES", MAIL_SERVER_MESSAGE_LIMIT)
        .with_env_var("MP_DISABLE_VERSION_CHECK", "true")
        .with_container_name(container_name)
        .with_label(MAIL_SERVER_CONTAINER_LABEL, container_name)
        .with_reuse(ReuseDirective::Always);
    let image = match allowed_recipients {
        Some(allowed_recipients) => {
            image.with_env_var("MP_SMTP_ALLOWED_RECIPIENTS", allowed_recipients)
        }
        None => image,
    };
    let container = image.start().await.unwrap_or_else(|error| {
        panic!(
            "email tests need a running Docker daemon (set DOCKER_HOST for OrbStack or colima): {error}"
        )
    });
    let host = container
        .get_host()
        .await
        .expect("read the mail server host")
        .to_string();
    let mail_server = MailServer {
        smtp_port: container
            .get_host_port_ipv4(MAIL_SERVER_SMTP_PORT)
            .await
            .expect("read the mail server SMTP port"),
        api_base_url: format!(
            "http://{host}:{}",
            container
                .get_host_port_ipv4(MAIL_SERVER_API_PORT)
                .await
                .expect("read the mail server API port")
        ),
        host,
    };
    wait_until_ready(&mail_server).await;
    drop(startup_lock);
    mail_server
}

async fn wait_until_ready(mail_server: &MailServer) {
    let readiness_url = format!("{}/readyz", mail_server.api_base_url);
    let deadline = Instant::now() + Duration::from_secs(MAIL_SERVER_READY_TIMEOUT_SECONDS);
    loop {
        let ready = matches!(
            reqwest::get(&readiness_url).await,
            Ok(response) if response.status() == StatusCode::OK
        );
        if ready {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the mail server at {readiness_url} did not become ready"
        );
        tokio::time::sleep(Duration::from_millis(MAIL_SERVER_READY_POLL_MILLISECONDS)).await;
    }
}
