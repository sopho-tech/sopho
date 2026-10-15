use strum::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum FailureKind {
    ServerError,
    MalformedJson,
}
