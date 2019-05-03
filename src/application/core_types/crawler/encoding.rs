use crate::charset::Charset;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Serialize)]
pub struct Encoding {
    pub input: Option<Charset>,
    pub output: Charset,
}
