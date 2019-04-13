pub mod request;
pub use self::request::{Method, Request};

#[macro_use]
pub mod response;
pub use self::response::{CrawlResult, CrawlerError};

pub mod crawler;
pub use self::crawler::Crawler;
