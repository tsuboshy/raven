use crate::application::core_types::crawler::encoding::Encoding;
use crate::application::core_types::crawler::request::Method;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq)]
pub struct RavenRequest {
    pub url: String,

    pub method: Method,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default)]
    pub vars: Vec<HashMap<String, Vec<String>>>,

    pub encoding: Option<Encoding>,

    #[serde(default = "default_timeout")]
    pub timeout_in_seconds: u8,

    #[serde(default)]
    pub max_retry: u8,

    #[serde(default)]
    pub params: Vec<HashMap<String, Vec<String>>>,
}

fn default_timeout() -> u8 {
    1
}
