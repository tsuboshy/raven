use super::super::logger::log_level::LogLevel;
use serde_derive::*;
use super::super::notify::Notify;
use super::request::Request;
use super::super::output::OutputMethod;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub name: String,

    pub request: Request,

    #[serde(default)]
    pub notify: Vec<Notify>,

    pub output: Vec<OutputMethod>,

    #[serde(default = "default_max_threads")]
    pub max_threads: u16,

    pub log: LogConfig
}

fn default_max_threads() -> u16 {
    1
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LogConfig {
    pub file_path: String,
    pub level: LogLevel
}