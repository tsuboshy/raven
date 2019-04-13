use crate::mime::Mime;
use crate::output::local_file::{write_to_local, FailedToWriteLocal, LocalFileRequest};
use crate::output::s3::{write_to_s3, S3WriteFileRequest, S3WriterError};
use serde_derive::*;
use std::{borrow::Cow, rc::Rc};

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
            OutputMethod::LocalFile { file_path } => *file_path = new_file_path,
            OutputMethod::AmazonS3 {
                ref mut object_key, ..
            } => *object_key = new_file_path,
        }
    }
}

/// Output trait supplies to output data to some storage such as local storage and amazon s3 and so on.
pub trait Output {
    fn output_crawled_data(
        crawled_data: &[u8],
        mime: Mime,
        method: OutputMethod,
    ) -> Result<(), OutputError> {
        default_impl_output_crawled_data(crawled_data, mime, method)
    }
}

pub fn default_impl_output_crawled_data(
    crawled_data: &[u8],
    content_type: Mime,
    method: OutputMethod,
) -> Result<(), OutputError> {
    match method {
        OutputMethod::LocalFile { file_path } => {
            let local_file_request = LocalFileRequest {
                file_path,
                content: crawled_data,
            };
            write_to_local(local_file_request)?;
            Ok(())
        }

        OutputMethod::AmazonS3 {
            region,
            bucket_name,
            object_key,
        } => {
            let s3_request = S3WriteFileRequest {
                region,
                bucket_name,
                object_key,
                content_type: &content_type,
                content: crawled_data,
            };
            write_to_s3(s3_request)?;
            Ok(())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum OutputError {
    FailedToWriteLocalFile(String),

    FailedToPutToS3(String),
}

impl From<FailedToWriteLocal> for OutputError {
    fn from(local_write_error: FailedToWriteLocal) -> Self {
        OutputError::FailedToWriteLocalFile(local_write_error.0)
    }
}

impl From<S3WriterError> for OutputError {
    fn from(s3_error: S3WriterError) -> Self {
        OutputError::FailedToPutToS3(s3_error.0)
    }
}
