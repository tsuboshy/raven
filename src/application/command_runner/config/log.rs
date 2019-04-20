use crate::application::core_types::logger::LogLevel;
use serde_derive::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct LogConfig {
    pub file_path: String,
    pub level: LogLevel,
}
