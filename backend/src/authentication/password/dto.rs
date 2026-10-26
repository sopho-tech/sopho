use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequestDto {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetDto {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordDto {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetRequestedDto {
    pub message: String,
}
