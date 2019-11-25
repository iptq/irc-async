use std::fmt;
use std::str::FromStr;

use super::command::{Command, ParseError};

#[derive(Debug)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: Command,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = self
            .prefix
            .as_ref()
            .map(|prefix| format!(":{} ", prefix))
            .unwrap_or_else(|| String::new());
        write!(f, "{}{}\r\n", prefix, self.command)
    }
}

impl FromStr for Message {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        println!("msg: {}", s);
        // if the first char is a ':', then there's a prefix
        let mut chars = s.chars();
        let (s2, prefix) = match chars.nth(0).filter(|c| *c == ':') {
            Some(_) => {
                let next_space = chars.position(|c| c == ' ').unwrap();
                (&s[next_space + 2..], Some(s[1..next_space + 1].to_owned()))
            }
            None => (s, None),
        };

        let command = Command::from_str(s2)?;
        Ok(Message { prefix, command })
    }
}
