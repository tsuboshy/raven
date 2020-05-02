use super::encoding::Encoding;
use crate::macros::HashMap;
use serde_derive::*;

#[derive(Debug, Eq, PartialEq, Serialize, Clone)]
pub struct CrawlerRequest {
    pub url: String,
    pub method: Method,
    pub header: HashMap<String, String>,
    pub encoding_setting: Option<Encoding>,
    pub timeout: u8,
    pub max_retry: u8,
    pub query_params: HashMap<String, String>,
    pub body_params: HashMap<String, String>,
    pub sleep: Option<u16>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Clone, Serialize)]
pub enum Method {
    Get,
    Post,
}
