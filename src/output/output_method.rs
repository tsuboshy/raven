use serde_derive::*;

/// this enum represents how to persist crawled content.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputMethod {
    /// write to local file
    LocalFile {
        file_path: String
    },

    /// write to amazon s3
    AmazonS3 {
        region: String,
        bucket_name: String,
        object_key: String
    }
    
}