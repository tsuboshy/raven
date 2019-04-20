#[derive(Debug, Eq, PartialEq)]
pub enum PersistError {
    FailedToWriteLocalFile(String),

    FailedToPutToS3(String),
}
