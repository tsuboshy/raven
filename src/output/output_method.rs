use serde_derive::*;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputMethod {
    LocalFile {
        file_path: String
    },

    AmazonS3 {
        region: String,
        bucket_name: String,
        object_key: String
    }
    
}