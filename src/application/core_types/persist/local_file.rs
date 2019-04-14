use crate::application::core_types::persist::PersistError;
use std::{
    error::Error,
    fs::{create_dir_all, OpenOptions},
    io::{BufWriter, Error as IOError, Write},
};

#[derive(Debug, Eq, PartialEq)]
pub struct FailedToWriteLocal(pub String);

impl From<FailedToWriteLocal> for PersistError {
    fn from(e: FailedToWriteLocal) -> Self {
        PersistError::FailedToWriteLocalFile(e.0)
    }
}

impl From<IOError> for FailedToWriteLocal {
    fn from(e: IOError) -> Self {
        FailedToWriteLocal(e.description().to_owned())
    }
}

pub fn write_to_local(file_path: &str, content: &[u8]) -> Result<(), FailedToWriteLocal> {
    let split_by_slash: Vec<&str> = file_path.split("/").collect();
    if let Some((_, dir_names)) = split_by_slash.split_last() {
        if dir_names.len() != 0 {
            create_dir_all(dir_names.join("/"))?;
        }
        let target_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)?;

        let mut writer = BufWriter::new(target_file);
        writer.write_all(content)?;
        writer.flush()?;
        Ok(())
    } else {
        return Err(FailedToWriteLocal("filepath name is empty".to_owned()));
    }
}

#[test]
fn test_write_to_local() {
    use std::fs::File;
    use std::io::{BufReader, Read};

    // arrange
    let test_file_path = "/var/tmp/application/test.txt";
    let test_content = "testだよーん";

    // act
    let result = write_to_local(test_file_path, test_content.as_bytes());

    // assert
    assert_eq!(Ok(()), result);

    let mut result_content: Vec<u8> = vec![];
    BufReader::new(File::open(test_file_path).unwrap())
        .read_to_end(&mut result_content)
        .unwrap();
    assert_eq!(test_content.as_bytes().to_owned(), result_content);
}
