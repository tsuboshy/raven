use crate::crawl::request::Encoding;
use crate::crawl::Method;
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
