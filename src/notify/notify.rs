use serde_derive::*;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Notify {

    Slack { 
        url: String,
        channel: String,
        mention: Option<String>
    }
}