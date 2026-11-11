use crate::email_template::constants::{
    EMAIL_DATE_FORMAT, EMAIL_LOGO_CONTENT_ID, EMAIL_LOGO_CONTENT_TYPE, EMAIL_LOGO_DATA_URI_PREFIX,
    EMAIL_LOGO_PNG, EMAIL_LOGO_SOURCE, EMAIL_TIME_FORMAT, EmailKind, NO_PASSWORD_SUBJECT,
    PASSWORD_CHANGED_SUBJECT, PASSWORD_RESET_SUBJECT, SAMPLE_FORGOT_PASSWORD_LINK,
    SAMPLE_INVITATION_EXPIRY_DAYS, SAMPLE_INVITATION_LINK, SAMPLE_INVITER_NAME,
    SAMPLE_PROVIDER_DISPLAY_NAME, SAMPLE_PROVIDER_NAMES, SAMPLE_RECIPIENT_NAME, SAMPLE_RESET_LINK,
    SAMPLE_ROLE_NAME, SAMPLE_SIGN_IN_LINK, SAMPLE_TENANT_NAME,
};
use crate::email_template::dto::{
    EmailInlineImage, InvitationEmail, NoPasswordEmail, PasswordChangedEmail, PasswordResetEmail,
    RenderedEmail, SignInMethodAddedEmail, SignInMethodRemovedEmail,
};
use crate::email_template::error::EmailTemplateError;
use crate::email_template::templates::{
    InvitationHtml, InvitationText, NoPasswordHtml, NoPasswordText, PasswordChangedHtml,
    PasswordChangedText, PasswordResetHtml, PasswordResetText, SignInMethodAddedHtml,
    SignInMethodAddedText, SignInMethodRemovedHtml, SignInMethodRemovedText,
};
use askama::Template;
use base64::prelude::{BASE64_STANDARD, Engine};
use chrono::{DateTime, Duration, Utc};
use std::str::FromStr;

pub trait EmailTemplate {
    fn render(&self) -> Result<RenderedEmail, EmailTemplateError>;
}

pub fn format_email_date(moment: DateTime<Utc>) -> String {
    moment.format(EMAIL_DATE_FORMAT).to_string()
}

pub fn format_email_time(moment: DateTime<Utc>) -> String {
    moment.format(EMAIL_TIME_FORMAT).to_string()
}

impl EmailTemplate for InvitationEmail {
    fn render(&self) -> Result<RenderedEmail, EmailTemplateError> {
        rendered_email(
            EmailKind::Invitation,
            format!(
                "{} invited you to {} on Sopho",
                self.inviter_name, self.tenant_name
            ),
            &InvitationText { email: self },
            &InvitationHtml { email: self },
        )
    }
}

impl EmailTemplate for PasswordResetEmail {
    fn render(&self) -> Result<RenderedEmail, EmailTemplateError> {
        rendered_email(
            EmailKind::PasswordReset,
            PASSWORD_RESET_SUBJECT.to_string(),
            &PasswordResetText { email: self },
            &PasswordResetHtml { email: self },
        )
    }
}

impl EmailTemplate for NoPasswordEmail {
    fn render(&self) -> Result<RenderedEmail, EmailTemplateError> {
        rendered_email(
            EmailKind::NoPassword,
            NO_PASSWORD_SUBJECT.to_string(),
            &NoPasswordText { email: self },
            &NoPasswordHtml { email: self },
        )
    }
}

impl EmailTemplate for PasswordChangedEmail {
    fn render(&self) -> Result<RenderedEmail, EmailTemplateError> {
        rendered_email(
            EmailKind::PasswordChanged,
            PASSWORD_CHANGED_SUBJECT.to_string(),
            &PasswordChangedText { email: self },
            &PasswordChangedHtml { email: self },
        )
    }
}

impl EmailTemplate for SignInMethodAddedEmail {
    fn render(&self) -> Result<RenderedEmail, EmailTemplateError> {
        rendered_email(
            EmailKind::SignInMethodAdded,
            format!(
                "{} sign-in was added to your Sopho account",
                self.provider_display_name
            ),
            &SignInMethodAddedText { email: self },
            &SignInMethodAddedHtml { email: self },
        )
    }
}

impl EmailTemplate for SignInMethodRemovedEmail {
    fn render(&self) -> Result<RenderedEmail, EmailTemplateError> {
        rendered_email(
            EmailKind::SignInMethodRemoved,
            format!(
                "{} sign-in was removed from your Sopho account",
                self.provider_display_name
            ),
            &SignInMethodRemovedText { email: self },
            &SignInMethodRemovedHtml { email: self },
        )
    }
}

fn rendered_email(
    kind: EmailKind,
    subject: String,
    text_template: &impl Template,
    html_template: &impl Template,
) -> Result<RenderedEmail, EmailTemplateError> {
    let render_error = |source| EmailTemplateError::Render { kind, source };
    Ok(RenderedEmail {
        kind,
        subject,
        text_body: text_template.render().map_err(render_error)?,
        html_body: html_template.render().map_err(render_error)?,
    })
}

pub fn inline_images_of(html_body: &str) -> Vec<EmailInlineImage> {
    if !html_body.contains(EMAIL_LOGO_SOURCE) {
        return Vec::new();
    }
    vec![EmailInlineImage {
        content_id: EMAIL_LOGO_CONTENT_ID,
        content_type: EMAIL_LOGO_CONTENT_TYPE,
        bytes: EMAIL_LOGO_PNG,
    }]
}

fn with_inline_images_embedded(rendered_email: RenderedEmail) -> RenderedEmail {
    let logo_data_uri = format!(
        "{EMAIL_LOGO_DATA_URI_PREFIX}{}",
        BASE64_STANDARD.encode(EMAIL_LOGO_PNG)
    );
    RenderedEmail {
        html_body: rendered_email
            .html_body
            .replace(EMAIL_LOGO_SOURCE, &logo_data_uri),
        ..rendered_email
    }
}

pub fn render_sample_emails() -> Result<Vec<RenderedEmail>, EmailTemplateError> {
    Ok(render_sample_emails_as_sent()?
        .into_iter()
        .map(with_inline_images_embedded)
        .collect())
}

fn render_sample_emails_as_sent() -> Result<Vec<RenderedEmail>, EmailTemplateError> {
    let now = Utc::now();
    let changed_at = format_email_time(now);
    Ok(vec![
        InvitationEmail {
            inviter_name: SAMPLE_INVITER_NAME.to_string(),
            tenant_name: SAMPLE_TENANT_NAME.to_string(),
            role_name: SAMPLE_ROLE_NAME.to_string(),
            invitation_link: SAMPLE_INVITATION_LINK.to_string(),
            expires_on: format_email_date(now + Duration::days(SAMPLE_INVITATION_EXPIRY_DAYS)),
        }
        .render()?,
        PasswordResetEmail {
            full_name: SAMPLE_RECIPIENT_NAME.to_string(),
            reset_link: SAMPLE_RESET_LINK.to_string(),
        }
        .render()?,
        NoPasswordEmail {
            full_name: SAMPLE_RECIPIENT_NAME.to_string(),
            provider_names: SAMPLE_PROVIDER_NAMES
                .iter()
                .map(|provider_name| provider_name.to_string())
                .collect(),
            sign_in_link: SAMPLE_SIGN_IN_LINK.to_string(),
        }
        .render()?,
        PasswordChangedEmail {
            full_name: SAMPLE_RECIPIENT_NAME.to_string(),
            changed_at: changed_at.clone(),
            forgot_password_link: SAMPLE_FORGOT_PASSWORD_LINK.to_string(),
        }
        .render()?,
        SignInMethodAddedEmail {
            full_name: SAMPLE_RECIPIENT_NAME.to_string(),
            provider_display_name: SAMPLE_PROVIDER_DISPLAY_NAME.to_string(),
            changed_at: changed_at.clone(),
        }
        .render()?,
        SignInMethodRemovedEmail {
            full_name: SAMPLE_RECIPIENT_NAME.to_string(),
            provider_display_name: SAMPLE_PROVIDER_DISPLAY_NAME.to_string(),
            changed_at,
        }
        .render()?,
    ])
}

pub fn email_kind_display_name(kind: &str) -> Option<&'static str> {
    EmailKind::from_str(kind)
        .ok()
        .map(|email_kind| email_kind.display_name())
}

#[cfg(test)]
mod tests {
    use super::{EmailTemplate, format_email_date, format_email_time};
    use crate::email_template::constants::EmailKind;
    use crate::email_template::dto::{
        InvitationEmail, NoPasswordEmail, PasswordChangedEmail, PasswordResetEmail, RenderedEmail,
        SignInMethodAddedEmail, SignInMethodRemovedEmail,
    };
    use chrono::{TimeZone, Utc};

    fn invitation_to(tenant_name: &str) -> InvitationEmail {
        InvitationEmail {
            inviter_name: "Ada Admin".to_string(),
            tenant_name: tenant_name.to_string(),
            role_name: "MEMBER".to_string(),
            invitation_link: "https://sopho.example.com/invitation#token=abc".to_string(),
            expires_on: "2 November 2026".to_string(),
        }
    }

    fn assert_link_in_both_versions(rendered: &RenderedEmail, link: &str) {
        assert!(rendered.text_body.contains(link), "{}", rendered.text_body);
        assert!(rendered.html_body.contains(link), "{}", rendered.html_body);
    }

    #[test]
    fn html_escapes_names_people_typed() {
        let rendered = invitation_to("<b>Acme</b>").render().unwrap();
        assert!(!rendered.html_body.contains("<b>Acme</b>"));
        assert!(rendered.html_body.contains("&#60;b&#62;Acme&#60;/b&#62;"));
        assert!(rendered.text_body.contains("<b>Acme</b>"));
        assert_eq!(
            rendered.subject,
            "Ada Admin invited you to <b>Acme</b> on Sopho"
        );
    }

    #[test]
    fn every_email_renders_both_versions_with_its_link() {
        let invitation = invitation_to("Acme").render().unwrap();
        assert_eq!(invitation.kind, EmailKind::Invitation);
        assert_link_in_both_versions(
            &invitation,
            "https://sopho.example.com/invitation#token=abc",
        );
        assert!(invitation.text_body.contains("2 November 2026"));

        let password_reset = PasswordResetEmail {
            full_name: "Bea Builder".to_string(),
            reset_link: "https://sopho.example.com/reset-password#token=def".to_string(),
        }
        .render()
        .unwrap();
        assert_eq!(password_reset.kind, EmailKind::PasswordReset);
        assert_eq!(password_reset.subject, "Reset your Sopho password");
        assert_link_in_both_versions(
            &password_reset,
            "https://sopho.example.com/reset-password#token=def",
        );

        let no_password = NoPasswordEmail {
            full_name: "Bea Builder".to_string(),
            provider_names: vec!["Google".to_string(), "Okta".to_string()],
            sign_in_link: "https://sopho.example.com/signin".to_string(),
        }
        .render()
        .unwrap();
        assert_eq!(no_password.kind, EmailKind::NoPassword);
        assert!(no_password.text_body.contains("Google, Okta"));
        assert_link_in_both_versions(&no_password, "https://sopho.example.com/signin");

        let password_changed = PasswordChangedEmail {
            full_name: "Bea Builder".to_string(),
            changed_at: "26 October 2026, 10:00 UTC".to_string(),
            forgot_password_link: "https://sopho.example.com/forgot-password".to_string(),
        }
        .render()
        .unwrap();
        assert_eq!(password_changed.kind, EmailKind::PasswordChanged);
        assert_link_in_both_versions(
            &password_changed,
            "https://sopho.example.com/forgot-password",
        );

        let added = SignInMethodAddedEmail {
            full_name: "Bea Builder".to_string(),
            provider_display_name: "Google".to_string(),
            changed_at: "26 October 2026, 10:00 UTC".to_string(),
        }
        .render()
        .unwrap();
        assert_eq!(
            added.subject,
            "Google sign-in was added to your Sopho account"
        );
        assert!(added.html_body.contains("Google sign-in was added"));

        let removed = SignInMethodRemovedEmail {
            full_name: "Bea Builder".to_string(),
            provider_display_name: "Google".to_string(),
            changed_at: "26 October 2026, 10:00 UTC".to_string(),
        }
        .render()
        .unwrap();
        assert_eq!(
            removed.subject,
            "Google sign-in was removed from your Sopho account"
        );
        assert!(removed.text_body.contains("26 October 2026, 10:00 UTC"));
    }

    #[test]
    fn dates_and_times_read_naturally() {
        let moment = Utc.with_ymd_and_hms(2026, 11, 2, 9, 5, 0).unwrap();
        assert_eq!(format_email_date(moment), "2 November 2026");
        assert_eq!(format_email_time(moment), "2 November 2026, 09:05 UTC");
    }
}
