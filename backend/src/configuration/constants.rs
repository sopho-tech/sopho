use strum::{Display, EnumString, IntoStaticStr, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum ConfigurationKey {
    AdminUserSetup,
    DemoDataSetup,
    DefaultTenantId,
}

impl ConfigurationKey {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn all() -> Vec<ConfigurationKey> {
        <Self as VariantArray>::VARIANTS.to_vec()
    }
}
