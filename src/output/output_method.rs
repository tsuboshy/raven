use serde_derive::*;

/// this enum represents how to persist crawled content.
#[derive(Debug, Deserialize, PartialEq, Clone, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OutputMethod {
    /// write to local file
    LocalFile { file_path: String },

    /// write to amazon s3
    AmazonS3 {
        region: String,
        bucket_name: String,
        object_key: String,
    },
}

impl OutputMethod {
    pub fn update_file_path(&mut self, new_file_path: String) {
        match self {
            OutputMethod::LocalFile { ref mut file_path } => *file_path = new_file_path,
            OutputMethod::AmazonS3 {
                ref mut object_key, ..
            } => *object_key = new_file_path,
        }
    }
}
