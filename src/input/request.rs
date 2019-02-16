use super::super::crawl::request::Method;
use serde_derive::*;
use std::collections::hash_map::HashMap;

#[derive(Debug, Deserialize, PartialEq)]
pub struct RavenRequest {
    pub url: String,

    pub method: Method,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default)]
    pub vars: Vec<HashMap<String, Vec<String>>>,

    #[serde(default = "utf8")]
    pub input_charset: String,

    #[serde(default = "utf8")]
    pub output_charset: String,

    #[serde(default = "default_timeout")]
    pub timeout_in_seconds: u8,

    #[serde(default)]
    pub max_retry: u8,

    #[serde(default)]
    pub params: Vec<HashMap<String, Vec<String>>>,
}

fn utf8() -> String {
    "UTF-8".to_string()
}

fn default_timeout() -> u8 {
    1
}
