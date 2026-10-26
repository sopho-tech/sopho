use crate::email_template::constants::EmailKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailTemplateError {
    #[error("Could not render the {kind} email: {source}")]
    Render {
        kind: EmailKind,
        #[source]
        source: askama::Error,
    },
}
