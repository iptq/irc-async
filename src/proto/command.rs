use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum Command {
    Nick(String),
    Notice(String, String),
    User(String, String, String),
    Unknown(String, Vec<String>),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidCommand(String),
    NoCommand,
    NotEnoughArgs,
}

macro_rules! write_command {
    ($writer:expr, $command:expr, [$($arg:expr),* $(,)?]) => {{
        write!($writer, "{} ", $command)
        $(.and_then(|_| write!($writer, "{} ", $arg)))*
    }};
    ($writer:expr, $command:expr, [$($arg:expr),* $(,)?], $final:expr) => {{
        write!($writer, "{} ", $command)
        $(.and_then(|_| write!($writer, "{} ", $arg)))*
        .and_then(|_| write!($writer, ":{}", $final))
    }};
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Nick(nickname) => write_command!(f, "NICK", [nickname,]),
            Command::Notice(msgtarget, message) => {
                write_command!(f, "NOTICE", [msgtarget], message)
            }
            Command::User(user, mode, realname) => {
                write_command!(f, "USER", [user, mode, "*"], realname)
            }
            Command::Unknown(command, args) => {
                write!(f, "{} ", command)?;
                if args.len() > 0 {
                    let mut arg_iter = args.iter().rev();
                    let last_elem = arg_iter.next().unwrap();
                    let mut arg_iter = arg_iter.rev();
                    for arg in arg_iter {
                        write!(f, "{} ", arg)?;
                    }
                    write!(f, ":{}", last_elem)?;
                }
                Ok(())
            }
        }
    }
}

macro_rules! parse_command {
    ($rest:expr, $variant:path => ($($arg:ident),* $(,)?)) => {
        {
            let mut iter = $rest.iter();
            $(
                let $arg = match iter.next() {
                    Some(value) => (*value).to_string(),
                    None => return Err(ParseError::NotEnoughArgs),
                };
            )*
            Ok($variant($($arg),*))
        }
    }
}

impl FromStr for Command {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut last_arg = s.split(":");
        let mut parts = last_arg.next().unwrap().split(" ");

        let command = match parts.next() {
            Some(command) => command,
            None => return Err(ParseError::NoCommand),
        };

        let mut rest = parts
            .filter_map(|c| if c.trim().is_empty() { None } else { Some(c) })
            .collect::<Vec<_>>();
        if let Some(last_arg) = last_arg.next() {
            rest.push(last_arg);
        }

        match command {
            "NICK" => parse_command!(rest, Command::Nick => (nickname)),
            "NOTICE" => parse_command!(rest, Command::Notice => (msgtarget, message)),
            "USER" => parse_command!(rest, Command::User => (user, mode, realname)),
            _ => Ok(Command::Unknown(
                command.to_owned(),
                rest.into_iter().map(|s| s.to_owned()).collect(),
            )),
        }
    }
}
