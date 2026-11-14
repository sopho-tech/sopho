use crate::common::AppState;
use crate::common::errors::ConfigurationError;
use crate::feature_flag::constants::{
    FEATURE_FLAG_DISABLED_VALUE, FEATURE_FLAG_ENABLED_VALUE, FeatureFlag,
};
use crate::feature_flag::dto::FeatureFlags;
use std::env::VarError;
use strum::VariantArray;
use tracing::info;

fn feature_flag_environment_variable(
    variable_name: &str,
) -> Result<Option<String>, ConfigurationError> {
    match std::env::var(variable_name) {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(value)) => Err(ConfigurationError::InvalidFeatureFlag {
            variable_name: variable_name.to_string(),
            value: value.to_string_lossy().into_owned(),
        }),
    }
}

pub fn feature_flags_from_environment() -> Result<FeatureFlags, ConfigurationError> {
    feature_flags_from_variables(feature_flag_environment_variable)
}

pub fn feature_flags_from_variables(
    read_variable: impl Fn(&str) -> Result<Option<String>, ConfigurationError>,
) -> Result<FeatureFlags, ConfigurationError> {
    let mut feature_flags = FeatureFlags::from_defaults();
    for feature_flag in FeatureFlag::VARIANTS.iter().copied() {
        let variable_name = feature_flag.environment_variable_name();
        let Some(value) = read_variable(&variable_name)? else {
            continue;
        };
        let is_enabled = match value.as_str() {
            FEATURE_FLAG_ENABLED_VALUE => true,
            FEATURE_FLAG_DISABLED_VALUE => false,
            _ => {
                return Err(ConfigurationError::InvalidFeatureFlag {
                    variable_name,
                    value,
                });
            }
        };
        feature_flags.set_enabled(feature_flag, is_enabled);
    }
    Ok(feature_flags)
}

pub fn is_feature_enabled(app_state: &AppState, feature_flag: FeatureFlag) -> bool {
    app_state.config.feature_flags.is_enabled(feature_flag)
}

pub fn log_feature_flags(feature_flags: &FeatureFlags) {
    for feature_flag in FeatureFlag::VARIANTS.iter().copied() {
        info!(
            "feature flag {} is {}",
            feature_flag.environment_variable_name(),
            if feature_flags.is_enabled(feature_flag) {
                "on"
            } else {
                "off"
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::feature_flags_from_variables;
    use crate::common::errors::ConfigurationError;
    use crate::feature_flag::constants::FeatureFlag;

    const SELF_SERVE_VARIABLE: &str = "SOPHO_FEATURE_SELF_SERVE_TENANT_CREATION";

    fn self_serve_flag_with_value(value: Option<&'static str>) -> Result<bool, ConfigurationError> {
        feature_flags_from_variables(|variable_name| {
            Ok((variable_name == SELF_SERVE_VARIABLE)
                .then_some(value)
                .flatten()
                .map(str::to_string))
        })
        .map(|feature_flags| feature_flags.is_enabled(FeatureFlag::SelfServeTenantCreation))
    }

    #[test]
    fn the_variable_name_carries_the_prefix_and_the_flag_name() {
        assert_eq!(
            FeatureFlag::SelfServeTenantCreation.environment_variable_name(),
            SELF_SERVE_VARIABLE
        );
    }

    #[test]
    fn an_unset_flag_takes_its_code_default() {
        assert_eq!(
            self_serve_flag_with_value(None).unwrap(),
            FeatureFlag::SelfServeTenantCreation.is_enabled_by_default()
        );
        assert!(!self_serve_flag_with_value(None).unwrap());
    }

    #[test]
    fn the_password_and_seeding_flags_are_on_unless_turned_off() {
        for (feature_flag, variable_name) in [
            (
                FeatureFlag::PasswordSignIn,
                "SOPHO_FEATURE_PASSWORD_SIGN_IN",
            ),
            (
                FeatureFlag::SeedPasswordAdministrator,
                "SOPHO_FEATURE_SEED_PASSWORD_ADMINISTRATOR",
            ),
            (FeatureFlag::DefaultTenant, "SOPHO_FEATURE_DEFAULT_TENANT"),
        ] {
            assert_eq!(feature_flag.environment_variable_name(), variable_name);
            assert!(
                feature_flags_from_variables(|_| Ok(None))
                    .unwrap()
                    .is_enabled(feature_flag)
            );
            assert!(
                !feature_flags_from_variables(|read_variable_name| {
                    Ok((read_variable_name == variable_name).then(|| "false".to_string()))
                })
                .unwrap()
                .is_enabled(feature_flag)
            );
        }
    }

    #[test]
    fn true_and_false_override_the_default() {
        assert!(self_serve_flag_with_value(Some("true")).unwrap());
        assert!(!self_serve_flag_with_value(Some("false")).unwrap());
    }

    #[test]
    fn any_other_value_is_refused_naming_the_variable() {
        for refused_value in ["yes", "True", " true ", "1", ""] {
            let error = self_serve_flag_with_value(Some(refused_value)).unwrap_err();
            assert!(
                matches!(
                    &error,
                    ConfigurationError::InvalidFeatureFlag { variable_name, value }
                        if variable_name == SELF_SERVE_VARIABLE && value == refused_value
                ),
                "{refused_value:?} was not refused"
            );
            assert!(error.to_string().contains(SELF_SERVE_VARIABLE));
        }
    }
}
