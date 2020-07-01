use crate::application::core_types::crawler::encoding::Encoding;
use crate::application::core_types::crawler::request::Method;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::{map::Map, Value};
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq)]
pub struct RavenRequest {
    pub url: String,

    pub method: Method,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default, deserialize_with = "deserialize_params")]
    pub vars: Vec<HashMap<String, Vec<String>>>,

    pub encoding: Option<Encoding>,

    #[serde(default = "default_timeout")]
    pub timeout_in_seconds: u8,

    #[serde(default)]
    pub max_retry: u8,

    #[serde(default, deserialize_with = "deserialize_params")]
    pub params: Vec<HashMap<String, Vec<String>>>,
}

fn default_timeout() -> u8 {
    1
}

fn deserialize_params<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<HashMap<String, Vec<String>>>, D::Error> {
    let json_node = <Value as Deserialize>::deserialize(deserializer)?;

    return match json_node {
        Value::Null => Ok(vec![]),
        Value::Bool(_) => Err(Error::custom("not support boolean")),
        Value::Number(_) => Err(Error::custom("not support number")),
        Value::String(_) => Err(Error::custom("not support string")),
        Value::Array(json_array) => {
            let mut result = vec![];
            for v in json_array {
                match v {
                    Value::Object(map) => {
                        let json = json_map_to_hash_map(map).map_err(|e| Error::custom(e))?;
                        result.push(json);
                    }
                    _ => return Err(Error::custom("invalid type: expected object")),
                }
            }
            return Ok(result);
        }
        Value::Object(map) => match json_map_to_hash_map(map) {
            Ok(hash_map) => Ok(vec![hash_map]),
            Err(msg) => Err(Error::custom(msg)),
        },
    };
}

fn json_map_to_hash_map(map: Map<String, Value>) -> Result<HashMap<String, Vec<String>>, String> {
    let mut result = HashMap::<String, Vec<String>>::new();
    for (k, v) in map {
        let strings = json_node_to_strings(v)?;
        result.insert(k, strings);
    }
    return Ok(result);
}

fn json_node_to_strings(json_node: Value) -> Result<Vec<String>, String> {
    return match json_node {
        Value::Null => Err("invalid type: null".to_owned()),
        Value::Bool(bool) => Ok(vec![bool.to_string()]),
        Value::Number(num) => Ok(vec![num.to_string()]),
        Value::String(str) => Ok(vec![str]),
        Value::Array(array) => {
            let mut result = vec![];
            for v in array {
                let mut to_strings = json_node_to_strings(v)?;
                result.append(&mut to_strings);
            }
            return Ok(result);
        }
        Value::Object(_) => Err("invalid type: object".to_owned()),
    };
}
