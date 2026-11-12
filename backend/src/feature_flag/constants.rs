use serde::Deserialize;
use strum::{Display, VariantArray};

pub const FEATURE_FLAG_VARIABLE_PREFIX: &str = "SOPHO_FEATURE_";
pub const FEATURE_FLAG_ENABLED_VALUE: &str = "true";
pub const FEATURE_FLAG_DISABLED_VALUE: &str = "false";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, VariantArray, Deserialize)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum FeatureFlag {
    SelfServeTenantCreation,
    PasswordSignIn,
    SeedPasswordAdministrator,
    DefaultTenant,
    InternalAddressConnections,
    BackgroundSuggestedQuestionRefresh,
}

impl FeatureFlag {
    pub fn environment_variable_name(self) -> String {
        format!("{FEATURE_FLAG_VARIABLE_PREFIX}{self}")
    }

    pub fn is_enabled_by_default(self) -> bool {
        match self {
            FeatureFlag::SelfServeTenantCreation => false,
            FeatureFlag::PasswordSignIn
            | FeatureFlag::SeedPasswordAdministrator
            | FeatureFlag::DefaultTenant
            | FeatureFlag::InternalAddressConnections
            | FeatureFlag::BackgroundSuggestedQuestionRefresh => true,
        }
    }
}
