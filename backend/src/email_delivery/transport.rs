use crate::common::EmailConfiguration;
use crate::common::constants::SmtpTls;
use crate::email_delivery::constants::SMTP_TIMED_OUT_MESSAGE;
use crate::email_template::dto::EmailInlineImage;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::time::Duration;

pub type EmailTransport = AsyncSmtpTransport<Tokio1Executor>;

pub enum SendFailure {
    Temporary { reason: String },
    Permanent { reason: String },
}

pub fn build_email_transport(
    configuration: &EmailConfiguration,
) -> Result<EmailTransport, lettre::transport::smtp::Error> {
    let transport_builder = match configuration.smtp_tls {
        SmtpTls::Tls => EmailTransport::relay(&configuration.smtp_host)?,
        SmtpTls::Starttls => EmailTransport::starttls_relay(&configuration.smtp_host)?,
        SmtpTls::Unencrypted => EmailTransport::builder_dangerous(&configuration.smtp_host),
    }
    .port(configuration.smtp_port)
    .timeout(Some(Duration::from_secs(
        configuration.smtp_timeout_seconds,
    )));
    let transport_builder = match &configuration.smtp_credentials {
        Some(smtp_credentials) => transport_builder.credentials(Credentials::new(
            smtp_credentials.username.clone(),
            smtp_credentials.password.clone(),
        )),
        None => transport_builder,
    };
    Ok(transport_builder.build())
}

pub fn build_message(
    from_address: &str,
    recipient: &str,
    subject: &str,
    text_body: String,
    html_body: String,
    inline_images: Vec<EmailInlineImage>,
) -> Result<Message, String> {
    let sender: Mailbox = from_address
        .parse()
        .map_err(|error| format!("Invalid sender {from_address:?}: {error}"))?;
    let receiver: Mailbox = recipient
        .parse()
        .map_err(|error| format!("Invalid recipient {recipient:?}: {error}"))?;
    let mut html_part = MultiPart::related().singlepart(SinglePart::html(html_body));
    for inline_image in inline_images {
        let content_type = ContentType::parse(inline_image.content_type).map_err(|error| {
            format!(
                "Invalid content type {:?} of the inline image {:?}: {error}",
                inline_image.content_type, inline_image.content_id
            )
        })?;
        html_part = html_part.singlepart(
            Attachment::new_inline(inline_image.content_id.to_string())
                .body(inline_image.bytes.to_vec(), content_type),
        );
    }
    Message::builder()
        .from(sender)
        .to(receiver)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(text_body))
                .multipart(html_part),
        )
        .map_err(|error| format!("Could not build the email: {error}"))
}

pub async fn send_message(
    transport: &EmailTransport,
    message: Message,
    send_timeout: Duration,
) -> Result<(), SendFailure> {
    match tokio::time::timeout(send_timeout, transport.send(message)).await {
        Err(_) => Err(SendFailure::Temporary {
            reason: SMTP_TIMED_OUT_MESSAGE.to_string(),
        }),
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) if error.is_permanent() => Err(SendFailure::Permanent {
            reason: error.to_string(),
        }),
        Ok(Err(error)) => Err(SendFailure::Temporary {
            reason: error.to_string(),
        }),
    }
}
