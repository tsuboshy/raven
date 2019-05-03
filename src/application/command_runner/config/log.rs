use crate::application::core_types::logger::LogLevel;
use serde_derive::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct LogConfig {
    pub file: FileLogConfig,
    pub elasticsearch: Option<EsConfig>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct FileLogConfig {
    pub path: String,
    pub level: LogLevel,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct EsConfig {
    pub endpoint: String,
}
