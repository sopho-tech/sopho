use crate::email_template::dto::{
    InvitationEmail, NoPasswordEmail, PasswordChangedEmail, PasswordResetEmail,
    SignInMethodAddedEmail, SignInMethodRemovedEmail,
};
use askama::Template;

#[derive(Template)]
#[template(path = "email/invitation.html")]
pub(super) struct InvitationHtml<'a> {
    pub(super) email: &'a InvitationEmail,
}

#[derive(Template)]
#[template(path = "email/invitation.txt")]
pub(super) struct InvitationText<'a> {
    pub(super) email: &'a InvitationEmail,
}

#[derive(Template)]
#[template(path = "email/password_reset.html")]
pub(super) struct PasswordResetHtml<'a> {
    pub(super) email: &'a PasswordResetEmail,
}

#[derive(Template)]
#[template(path = "email/password_reset.txt")]
pub(super) struct PasswordResetText<'a> {
    pub(super) email: &'a PasswordResetEmail,
}

#[derive(Template)]
#[template(path = "email/no_password.html")]
pub(super) struct NoPasswordHtml<'a> {
    pub(super) email: &'a NoPasswordEmail,
}

#[derive(Template)]
#[template(path = "email/no_password.txt")]
pub(super) struct NoPasswordText<'a> {
    pub(super) email: &'a NoPasswordEmail,
}

#[derive(Template)]
#[template(path = "email/password_changed.html")]
pub(super) struct PasswordChangedHtml<'a> {
    pub(super) email: &'a PasswordChangedEmail,
}

#[derive(Template)]
#[template(path = "email/password_changed.txt")]
pub(super) struct PasswordChangedText<'a> {
    pub(super) email: &'a PasswordChangedEmail,
}

#[derive(Template)]
#[template(path = "email/sign_in_method_added.html")]
pub(super) struct SignInMethodAddedHtml<'a> {
    pub(super) email: &'a SignInMethodAddedEmail,
}

#[derive(Template)]
#[template(path = "email/sign_in_method_added.txt")]
pub(super) struct SignInMethodAddedText<'a> {
    pub(super) email: &'a SignInMethodAddedEmail,
}

#[derive(Template)]
#[template(path = "email/sign_in_method_removed.html")]
pub(super) struct SignInMethodRemovedHtml<'a> {
    pub(super) email: &'a SignInMethodRemovedEmail,
}

#[derive(Template)]
#[template(path = "email/sign_in_method_removed.txt")]
pub(super) struct SignInMethodRemovedText<'a> {
    pub(super) email: &'a SignInMethodRemovedEmail,
}
