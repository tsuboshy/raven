pub mod log_level;
pub use log_level::LogLevel;
pub use log_level::LogLevel::*;
use std::path::Path;

pub fn init_file_logger(path: &str) {}
