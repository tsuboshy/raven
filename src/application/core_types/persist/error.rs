use serde_derive::Serialize;
use std::fmt::{Display, Formatter, Error};

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum PersistError {
    FailedToWriteLocalFile(String),

    FailedToPutToS3(String),
}

impl Display for PersistError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            PersistError::FailedToWriteLocalFile(e) => write!(f, "failed to write to local file: {}", e),
            PersistError::FailedToPutToS3(e) => write!(f, "failed to put to s3: {}", e)
        }
    }
}


impl std::error::Error for PersistError {}