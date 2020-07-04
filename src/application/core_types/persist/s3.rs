use std::default::Default;
use std::str::FromStr;
use std::time::Duration;

use rusoto_core::ByteStream;
pub use rusoto_core::Region;
use rusoto_s3::{PutObjectError, PutObjectOutput, PutObjectRequest, S3Client, S3};

use super::PersistError;
use crate::mime::Mime;

pub fn write_to_s3(request: S3WriteFileRequest) -> Result<(), S3WriterError> {
    let typed_region = match Region::from_str(&request.region) {
        Ok(parsed) => parsed,
        Err(parse_error) => return Err(S3WriterError(parse_error.to_string())),
    };
    let client = S3Client::new(typed_region);
    let mut retry_count = 5;
    loop {
        let request = request.to_put_object_request();

        let result: Result<PutObjectOutput, PutObjectError> = client
            .put_object(request)
            .with_timeout(Duration::from_secs(10))
            .sync();

        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                if let PutObjectError::HttpDispatch(_) = e {
                    if retry_count == 0 {
                        return Err(S3WriterError(e.to_string()));
                    } else {
                        retry_count -= 1;
                        continue;
                    }
                } else {
                    return Err(S3WriterError(e.to_string()));
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct S3WriteFileRequest<'a> {
    pub region: String,
    pub bucket_name: String,
    pub object_key: String,
    pub content_type: &'a Mime,
    pub content: &'a [u8],
}

impl<'a> S3WriteFileRequest<'a> {
    fn to_put_object_request(&self) -> PutObjectRequest {
        PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: self.object_key.to_owned(),
            content_type: Some(self.content_type.to_string()),
            body: Some(ByteStream::from(self.content.to_owned())),
            ..Default::default()
        }
    }
}
#[derive(Debug, Eq, PartialEq)]
pub struct S3WriterError(pub String);

impl From<S3WriterError> for PersistError {
    fn from(e: S3WriterError) -> Self {
        PersistError::FailedToPutToS3(e.0)
    }
}

#[test]
#[ignore]
fn s3_upload_test() {
    use crate::charset::Charset;
    use crate::mime::TextMime;
    let test_strings = "テストだよーん";
    let s3_request = S3WriteFileRequest {
        region: "ap-northeast-1".to_owned(),
        bucket_name: "crow-dev".to_owned(),
        object_key: "test_raven1.txt".to_owned(),
        content_type: &Mime::Text {
            charset: Some(Charset::Utf8),
            text_type: TextMime::TextPlain,
        },
        content: test_strings.as_bytes(),
    };
    let result = write_to_s3(s3_request).unwrap();
    println!("{:?}", result);
    assert_eq!(1, 1);
}
