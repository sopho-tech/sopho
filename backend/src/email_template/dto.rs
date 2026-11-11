use crate::email_template::constants::EmailKind;

pub struct RenderedEmail {
    pub kind: EmailKind,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
}

pub struct InvitationEmail {
    pub inviter_name: String,
    pub tenant_name: String,
    pub role_name: String,
    pub invitation_link: String,
    pub expires_on: String,
}

pub struct PasswordResetEmail {
    pub full_name: String,
    pub reset_link: String,
}

pub struct NoPasswordEmail {
    pub full_name: String,
    pub provider_names: Vec<String>,
    pub sign_in_link: String,
}

pub struct PasswordChangedEmail {
    pub full_name: String,
    pub changed_at: String,
    pub forgot_password_link: String,
}

pub struct SignInMethodAddedEmail {
    pub full_name: String,
    pub provider_display_name: String,
    pub changed_at: String,
}

pub struct SignInMethodRemovedEmail {
    pub full_name: String,
    pub provider_display_name: String,
    pub changed_at: String,
}
