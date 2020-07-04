use std::collections::HashMap;

use crate::charset::Charset;
use crate::mime::Mime;
use chrono::{DateTime, Local};
use serde_derive::Serialize;
use std::fmt::{Display, Error, Formatter};
use std::string::ToString;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum CrawlerError {
    ClientError(CrawlerResult),

    ServerError(CrawlerResult),

    TimeoutError {
        timeout_second: u8,
        retry_count: u8,
    },

    CharsetConversionError {
        error_detail: String,
        crawler_result: CrawlerResult,
    },

    OtherError {
        error_detail: String,
    },
}

impl CrawlerError {
    pub fn err_code(&self) -> u16 {
        match self {
            CrawlerError::ClientError(_) => 400,
            CrawlerError::ServerError(_) => 500,
            CrawlerError::TimeoutError { .. } => 600,
            CrawlerError::CharsetConversionError { .. } => 700,
            CrawlerError::OtherError { .. } => 800,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CrawlerError::ClientError(_) => "client error",
            CrawlerError::ServerError(_) => "server error",
            CrawlerError::TimeoutError { .. } => "timeout",
            CrawlerError::CharsetConversionError { .. } => "charset conversion failed",
            CrawlerError::OtherError { error_detail: _ } => "other error",
        }
    }
}

impl Display for CrawlerError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        static NOT_UTF8_MSG: &'static str = "(not utf-8)";
        match self {
            CrawlerError::ClientError(result) => write!(
                f,
                "client_error: {}",
                String::from_utf8(result.response_body.clone()).unwrap_or(NOT_UTF8_MSG.to_owned())
            ),
            CrawlerError::ServerError(result) => write!(
                f,
                "server_error: {}",
                String::from_utf8(result.response_body.clone()).unwrap_or(NOT_UTF8_MSG.to_owned())
            ),
            CrawlerError::TimeoutError {
                timeout_second,
                retry_count,
            } => write!(
                f,
                "request timeout: {} seconds (retry: {})",
                timeout_second, retry_count
            ),
            CrawlerError::CharsetConversionError { error_detail, .. } => {
                write!(f, "failed to convert charset: {}", error_detail,)
            }
            CrawlerError::OtherError { error_detail } => {
                write!(f, "unexpected error: {}", error_detail)
            }
        }
    }
}

#[macro_export]
macro_rules! other_error {
    ( $format:tt, $( $obj:tt ),* ) => {
        $crate::application::core_types::crawler::result::CrawlerError::OtherError {
            error_detail: format!($format, $( $obj ),* )
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct CrawlerResult {
    pub response_status: u16,
    pub response_header: HashMap<String, String>,
    pub response_body: Vec<u8>,
    pub mills_takes_to_complete_to_request: i64,
    pub retry_count: u8,
    pub response_content_type: Mime,
    pub crawl_date: DateTime<Local>,
}

pub fn get_result_code(result: &Result<CrawlerResult, CrawlerError>) -> u16 {
    match result {
        Ok(_) => 200,
        Err(crawler_error) => crawler_error.err_code(),
    }
}

pub fn get_result_message(result: &Result<CrawlerResult, CrawlerError>) -> String {
    match result {
        Ok(_) => "success".to_owned(),
        Err(crawler_error) => crawler_error.to_string(),
    }
}

impl CrawlerResult {
    pub fn convert_response_encoding_if_has_text_mime_type(&mut self, to: Charset) {
        match &mut self.response_content_type {
            Mime::Text {
                charset: Some(charset),
                ..
            } => {
                self.response_body = charset.convert_to(&to, &self.response_body);
                *charset = to;
            }
            _ => (),
        }
    }

    pub fn has_same_charset(&self, charset: &Charset) -> bool {
        self.response_content_type.has_same_charset(charset)
    }
}
