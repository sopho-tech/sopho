use serde::Serialize;
use strum::{Display, EnumString, IntoStaticStr, VariantArray};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Display, EnumString, IntoStaticStr, VariantArray,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    Active,
    Deactivated,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn all() -> Vec<UserStatus> {
        <Self as VariantArray>::VARIANTS.to_vec()
    }
}
