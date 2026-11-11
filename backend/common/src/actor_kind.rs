use serde::Serialize;
use strum::{Display, EnumString, IntoStaticStr, VariantArray};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Display, EnumString, IntoStaticStr, VariantArray,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActorKind {
    Human,
    Ai,
    System,
}

impl ActorKind {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn from_actor_kind_name(value: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(value)
            .map_err(|_| format!("Invalid actor kind: {}", value))
    }

    pub fn all() -> Vec<ActorKind> {
        <Self as VariantArray>::VARIANTS.to_vec()
    }
}
