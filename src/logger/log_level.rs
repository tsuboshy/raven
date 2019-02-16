use serde::de::{Deserialize, Deserializer, Error, Unexpected, Visitor};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn to_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<LogLevel, String> {
        let in_lower_case = s.to_lowercase();
        match &*in_lower_case {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            invalid => Err([
                "invalid value for LogLevel: ".to_string(),
                invalid.to_string(),
            ]
            .concat()),
        }
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<LogLevel, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(LogLevelVisitor)
    }
}

struct LogLevelVisitor;
impl<'de> Visitor<'de> for LogLevelVisitor {
    type Value = LogLevel;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            r#"one of "trace", "debug", "info", "warn", "error" (case insensitive) ."#
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<LogLevel, E>
    where
        E: Error,
    {
        match LogLevel::from_str(v) {
            Ok(log_level) => Ok(log_level),
            Err(_) => Err(E::invalid_type(Unexpected::Str(v), &self)),
        }
    }
}
