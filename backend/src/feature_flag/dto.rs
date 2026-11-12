use crate::feature_flag::constants::FeatureFlag;
use serde::Deserialize;
use std::collections::HashSet;
use strum::VariantArray;

#[derive(Clone, Debug, Deserialize)]
pub struct FeatureFlags {
    enabled_feature_flags: HashSet<FeatureFlag>,
}

impl FeatureFlags {
    pub fn from_defaults() -> Self {
        Self {
            enabled_feature_flags: FeatureFlag::VARIANTS
                .iter()
                .copied()
                .filter(|feature_flag| feature_flag.is_enabled_by_default())
                .collect(),
        }
    }

    pub fn set_enabled(&mut self, feature_flag: FeatureFlag, is_enabled: bool) {
        if is_enabled {
            self.enabled_feature_flags.insert(feature_flag);
        } else {
            self.enabled_feature_flags.remove(&feature_flag);
        }
    }

    pub fn is_enabled(&self, feature_flag: FeatureFlag) -> bool {
        self.enabled_feature_flags.contains(&feature_flag)
    }
}
