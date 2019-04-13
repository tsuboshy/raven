use crate::charset::*;
use crate::output::OutputMethod;
use serde_derive::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub url: String,
    pub method: Method,
    pub header: HashMap<String, String>,
    pub output_methods: Vec<OutputMethod>,
    pub encoding: Option<Encoding>,
    pub timeout: u8,
    pub max_retry: u8,
    pub val_map: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body_params: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Clone)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Encoding {
    pub input: Option<Charset>,
    pub output: Charset,
}
