use serde_derive::*;

/// this module supplies how to persist crawled data.

#[derive(Debug, Deserialize, PartialEq, Clone, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistMethod {
    /// write to local file
    LocalFile { file_path: String },

    /// write to amazon s3
    AmazonS3 {
        region: String,
        bucket_name: String,
        object_key: String,
    },
}

impl PersistMethod {
    pub fn update_file_path(&mut self, new_file_path: String) {
        match self {
            PersistMethod::LocalFile { file_path } => *file_path = new_file_path,
            PersistMethod::AmazonS3 {
                ref mut object_key, ..
            } => *object_key = new_file_path,
        };
    }

    pub fn get_file_name(&self) -> &str {
        match self {
            PersistMethod::LocalFile { file_path } => file_path,
            PersistMethod::AmazonS3 { object_key, .. } => object_key,
        }
    }
}
