use crate::application::core_types::crawler::result::{get_result_code, get_result_message};
use crate::application::core_types::crawler::{
    CrawlerError, CrawlerError::*, CrawlerRequest, CrawlerResult,
};
use crate::application::core_types::log::elastic_search::EsDocument;
use chrono::{DateTime, Local};
use rs_es::operations::mapping::Settings;
use serde_derive::Serialize;
use serde_json::Value;
use std::str::FromStr;

use crate::hostname::get_hostname;
use crate::serde_dateformat::yyyy_mm_dd_hh_ii_ss_z;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct CrawlerMetrics {
    pub crawler_name: String,
    pub result_code: u16,
    pub result_message: &'static str,
    pub request_duration_millis: Option<i64>,
    pub error_detail: Option<String>,
    pub request: CrawlerRequest,
    pub retry_count: u8,
    #[serde(with = "yyyy_mm_dd_hh_ii_ss_z")]
    pub crawled_date: DateTime<Local>,
    pub hostname: String,

    #[serde(skip_serializing)]
    index_name: String,
}

impl CrawlerMetrics {
    pub fn new(
        crawler_name: &str,
        request: CrawlerRequest,
        result: &Result<CrawlerResult, CrawlerError>,
    ) -> CrawlerMetrics {
        let result_code = get_result_code(&result);
        let result_message = get_result_message(&result);

        let request_duration_millis: Option<i64> = match &result {
            Ok(success) => Some(success.mills_takes_to_complete_to_request),
            Err(e) => match e {
                ClientError(client_error) => Some(client_error.mills_takes_to_complete_to_request),
                ServerError(server_error) => Some(server_error.mills_takes_to_complete_to_request),
                TimeoutError { timeout_second, .. } => Some(timeout_second.clone() as i64),
                CharsetConversionError { crawler_result, .. } => {
                    Some(crawler_result.mills_takes_to_complete_to_request)
                }
                OtherError { .. } => None,
            },
        };
        let retry_count = match &result {
            Ok(success) => success.retry_count,
            Err(e) => match e {
                ClientError(client_error) => client_error.retry_count.clone(),
                ServerError(server_error) => server_error.retry_count.clone(),
                TimeoutError { retry_count, .. } => retry_count.clone(),
                CharsetConversionError { crawler_result, .. } => crawler_result.retry_count.clone(),
                OtherError { .. } => 0,
            },
        };

        let crawled_date = match &result {
            Ok(success) => success.crawl_date,
            Err(e) => match e {
                ClientError(client_error) => client_error.crawl_date,
                ServerError(server_error) => server_error.crawl_date,
                TimeoutError { .. } => Local::now(),
                CharsetConversionError { crawler_result, .. } => crawler_result.crawl_date,
                OtherError { .. } => Local::now(),
            },
        };

        let index_name = crawled_date.format("raven-crawler-%Y-%m-%d").to_string();

        let error_detail = result.as_ref().err().map(|e| e.to_string());

        CrawlerMetrics {
            crawler_name: crawler_name.to_owned(),
            result_code,
            result_message,
            request_duration_millis,
            error_detail,
            request,
            retry_count,
            crawled_date,
            index_name,
            hostname: get_hostname().to_owned(),
        }
    }
}

impl<'a> EsDocument<'a> for CrawlerMetrics {
    fn elastic_search_template() -> &'static Value {
        static MAPPING_JSON: &'static str = r#"
            {
                "index_patterns": ["raven-crawler-*"],

                "mappings": {
                    "raven-crawler-metrics": {
                        "properties": {
                            "name": { "type": "text" },
        
                            "result_code": { "type": "integer" },
        
                            "result_message": { "type": "text" },
        
                            "request_duration_millis": { "type": "long" },
        
                            "request": {
                                "type": "nested",
        
                                "properties": {
                                    "url": { "type": "text" },
        
                                    "method": { "type": "keyword" },
        
                                    "header": { "type": "object" },
        
                                    "encoding_setting": { "type": "object" },
        
                                    "timeout": { "type": "integer" },
        
                                    "retry_max": { "type": "integer" },
        
                                    "query_params":{ "type": "object" },
        
                                    "body_params":{ "type": "object" }
                                }
                            },
        
                            "error_detail": { "type": "text" },
        
                            "hostname": { "type": "keyword" },
        
                            "retry_count": { "type": "integer" },
        
                            "crawled_date": {
                                "type": "date",
                                "format": "yyyy-MM-dd HH:mm:ssZZ"
                            }
                        }
                    }
                }
            }
        "#;

        lazy_static! {
            static ref MAPPING: Value = Value::from_str(MAPPING_JSON).unwrap();
        };

        &*MAPPING
    }

    fn elastic_search_typename() -> &'a str {
        "raven-crawler-metrics"
    }

    fn elastic_search_settings() -> Option<&'a Settings> {
        None
    }

    fn elastic_search_index_name(&self) -> &str {
        &self.index_name
    }
}

#[test]
fn mapping_json_test() {
    /// if MAPPING_JSON is malformed, this test causes panic.
    println!("{}", CrawlerMetrics::elastic_search_template());
}
