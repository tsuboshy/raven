use std::collections::HashMap;

use crate::mime::Mime;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CrawlerError {
    ClientError(CrawlerResult),

    ServerError(CrawlerResult),

    TimeoutError { timeout_second: u8, retry_count: u8 },

    CharsetConversionError { error_detail: String },

    OtherError { error_detail: String },
}

#[macro_export]
macro_rules! other_error {
    ( $format:tt, $( $obj:tt ),* ) => {
        $crate::application::core_types::crawler::result::CrawlerError::OtherError {
            error_detail: format!($format, $( $obj ),* )
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CrawlerResult {
    pub response_status: u16,
    pub response_header: HashMap<String, String>,
    pub response_body: Vec<u8>,
    pub mills_takes_to_complete_to_request: i64,
    pub retry_count: u8,
    pub response_content_type: Mime,
}
