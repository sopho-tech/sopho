use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sender {
    Human,
    Assistant,
}

impl fmt::Display for Sender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sender::Human => write!(f, "HUMAN"),
            Sender::Assistant => write!(f, "ASSISTANT"),
        }
    }
}

impl FromStr for Sender {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HUMAN" | "USER" => Ok(Sender::Human),
            "ASSISTANT" => Ok(Sender::Assistant),
            _ => Err(format!("Invalid sender: {}", s)),
        }
    }
}
