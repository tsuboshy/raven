use super::{
    error::PersistError,
    local_file::write_to_local,
    persist_method::PersistMethod,
    s3::{write_to_s3, S3WriteFileRequest},
};
use crate::mime::Mime;

pub trait Persist {
    fn persist_data(
        &self,
        method: &PersistMethod,
        data: &[u8],
        mime: &Mime,
    ) -> Result<(), PersistError> {
        persist_default_impl(method, data, mime)
    }
}

pub fn persist_default_impl(
    method: &PersistMethod,
    content: &[u8],
    mime: &Mime,
) -> Result<(), PersistError> {
    match method {
        PersistMethod::LocalFile { file_path } => {
            write_to_local(&file_path, content)?;
            Ok(())
        }

        PersistMethod::AmazonS3 {
            region,
            bucket_name,
            object_key,
        } => {
            let s3_request = S3WriteFileRequest {
                region: region.to_owned(),
                bucket_name: bucket_name.to_owned(),
                object_key: object_key.to_owned(),
                content_type: mime,
                content,
            };
            write_to_s3(s3_request)?;
            Ok(())
        }
    }
}
