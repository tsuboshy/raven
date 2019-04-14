use crate::charset::Charset;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Encoding {
    pub input: Option<Charset>,
    pub output: Charset,
}
