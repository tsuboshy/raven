use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CrawlerError {
    ClientError(Response),

    ServerError(Response),

    TimeoutError { timeout_second: u8, retry_count: u8 },

    OtherError { error_detail: String },
}

#[macro_export]
macro_rules! other_error {
    ( $format:tt, $( $obj:tt ),* ) => {
        $crate::crawl::response::CrawlerError::OtherError {
            error_detail: format!($format, $( $obj ),* )
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response {
    pub status: u16,
    pub header: HashMap<String, Vec<u8>>,
    pub body: Vec<u8>,
    pub mills_takes_to_complete_to_request: i64,
    pub retry_count: u8,
}
