use strum::{AsRefStr, Display, EnumIter, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, AsRefStr, Display, EnumIter, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Provider {
    Anthropic,
    Openai,
}
