mod log_level;

pub use self::log_level::*;
use std::error::Error;

pub fn write_error_log_if_err<T, E: Error>(label: &str, result: Result<T, E>) -> Result<T, E> {
    if let Err(err) = &result {
        error!("{}: {}", label, err.to_string());
    }
    result
}

pub fn write_warn_log_if_err<T, E: Error>(label: &str, result: Result<T, E>) -> Result<T, E> {
    if let Err(err) = &result {
        warn!("{}: {}", label, err.to_string());
    }
    result
}
