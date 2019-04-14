mod log_level;

pub use self::log_level::*;
use std::error::Error;
use std::fmt::Debug;

pub trait Logger {
    fn log(&self, level: LogLevel, message: &str);

    fn log_debug<T: Debug>(&self, label: &str, object: &T) {
        self.log(LogLevel::Debug, &format!("{}: {:?}", label, object));
    }

    fn log_info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    fn log_warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    fn log_warn_if_err<T, E: Error>(&self, result: Result<T, E>) -> Result<T, E> {
        if let Err(err) = &result {
            self.log_warn(err.description());
        }
        result
    }

    fn log_error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    fn log_error_if_err<T, E: Error>(&self, result: Result<T, E>) -> Result<T, E> {
        if let Err(err) = &result {
            self.log_error(err.description());
        }
        result
    }
}
