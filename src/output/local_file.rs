use std::error::Error;
use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::io::{BufWriter, Error as IOError, Read, Write};

#[derive(Debug, Eq, PartialEq)]
pub struct LocalFileRequest<'a> {
    pub file_path: String,
    pub content: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct FailedToWriteLocal(pub String);

impl From<IOError> for FailedToWriteLocal {
    fn from(e: IOError) -> Self {
        FailedToWriteLocal(e.description().to_owned())
    }
}

pub fn write_to_local(request: LocalFileRequest) -> Result<(), FailedToWriteLocal> {
    let split_by_slash: Vec<&str> = request.file_path.split("/").collect();
    if let Some((_, dir_names)) = split_by_slash.split_last() {
        if dir_names.len() != 0 {
            create_dir_all(dir_names.join("/"))?;
        }
        let target_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(request.file_path)?;

        let mut writer = BufWriter::new(target_file);
        writer.write_all(request.content)?;
        writer.flush()?;
        Ok(())
    } else {
        return Err(FailedToWriteLocal("filepath name is empty".to_owned()));
    }
}

#[test]
fn test_write_to_local() {
    use std::fs::File;
    use std::io::BufReader;
    // arrange
    let test_file_path = "/var/tmp/raven/test.txt";
    let test_content = "testだよーん";
    let local_request = LocalFileRequest {
        file_path: test_file_path.to_owned(),
        content: test_content.as_bytes(),
    };

    // act
    let result = write_to_local(local_request);

    // assert
    assert_eq!(Ok(()), result);

    let mut result_content: Vec<u8> = vec![];
    BufReader::new(File::open(test_file_path).unwrap())
        .read_to_end(&mut result_content)
        .unwrap();
    assert_eq!(test_content.as_bytes().to_owned(), result_content);
}
