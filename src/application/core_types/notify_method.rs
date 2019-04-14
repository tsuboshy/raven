use super::logger::{LogLevel, Logger};
use serde_derive::*;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyMethod {
    Slack {
        url: String,
        channel: String,
        mention: Option<String>,
    },
}

pub trait Notify {
    fn notify(&self, level: LogLevel, message: &str) -> Result<(), NotifyError>;

    fn notify_error(&self, message: &str) -> Result<(), NotifyError> {
        self.notify(LogLevel::Error, message)
    }

    fn notify_warn(&self, message: &str) -> Result<(), NotifyError> {
        self.notify(LogLevel::Warn, message)
    }

    fn notify_info(&self, message: &str) -> Result<(), NotifyError> {
        self.notify(LogLevel::Info, message)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct NotifyError(pub String);

impl Display for NotifyError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "failed to notify: {}", self.0)
    }
}

impl Error for NotifyError {}
