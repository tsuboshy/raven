use super::logger::LogLevel;
use crate::application::core_types::logger::write_error_log_if_err;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub trait Notify {
    fn notify(&self, level: LogLevel, label: &str, message: &str) -> Result<(), NotifyError>;

    fn notify_error(&self, label: &str, message: &str) -> Result<(), NotifyError> {
        write_error_log_if_err(
            "failed to notify",
            self.notify(LogLevel::Error, label, message),
        )
    }

    fn notify_warn(&self, label: &str, message: &str) -> Result<(), NotifyError> {
        write_error_log_if_err(
            "failed to notify",
            self.notify(LogLevel::Warn, label, message),
        )
    }

    fn notify_info(&self, label: &str, message: &str) -> Result<(), NotifyError> {
        write_error_log_if_err(
            "failed to notify",
            self.notify(LogLevel::Info, label, message),
        )
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
