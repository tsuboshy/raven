use crate::application::core_types::{crawler::CrawlerError, persist::PersistError};
use serde_derive::Serialize;

#[derive(PartialEq, Debug, Serialize)]
pub enum CrawlTaskError {
    CrawlerFailed(CrawlerError),

    PersistFailed {
        crawler_result: CrawlerResult,
        persist_errors: Vec<PersistError>,
        persist_duration_millis: i64,
    },
}

use self::CrawlTaskError::*;
use crate::application::core_types::crawler::CrawlerResult;
use rs_es::util::StrJoin;
use std::fmt::{Display, Error, Formatter};

impl From<CrawlerError> for CrawlTaskError {
    fn from(e: CrawlerError) -> Self {
        CrawlerFailed(e)
    }
}

impl CrawlTaskError {
    pub fn err_code(&self) -> u16 {
        match self {
            CrawlerFailed(crawler_error) => crawler_error.err_code(),
            PersistFailed { .. } => 1000,
        }
    }

    pub fn err_label(&self) -> &'static str {
        match self {
            CrawlerFailed(crawler_error) => crawler_error.description(),
            PersistFailed { .. } => "persist failed",
        }
    }
}

impl Display for CrawlTaskError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            CrawlerFailed(error) => write!(f, "failed to crawl: {}", error),
            PersistFailed { persist_errors, .. } => write!(
                f,
                "failed to persist: {}",
                persist_errors
                    .iter()
                    .map(|error| error.to_string())
                    .join(", ")
            ),
        }
    }
}
