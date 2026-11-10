use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr, VariantArray};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    IntoStaticStr,
    VariantArray,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmailStatus {
    Pending,
    Sending,
    Sent,
    Failed,
    Cancelled,
}

impl EmailStatus {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn all() -> Vec<EmailStatus> {
        <Self as VariantArray>::VARIANTS.to_vec()
    }
}
