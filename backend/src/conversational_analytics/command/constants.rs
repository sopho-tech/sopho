#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Canvas,
}

impl Command {
    pub fn parse(name: &str) -> Option<Command> {
        match name.trim().to_lowercase().as_str() {
            "canvas" => Some(Command::Canvas),
            _ => None,
        }
    }
}

pub fn parse_dedup(names: &[String]) -> Vec<Command> {
    let mut seen: Vec<Command> = Vec::new();
    for name in names {
        if let Some(cmd) = Command::parse(name) {
            if !seen.contains(&cmd) {
                seen.push(cmd);
            }
        }
    }
    seen
}
