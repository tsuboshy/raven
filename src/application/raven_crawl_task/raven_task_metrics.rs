use crate::application::core_types::crawler::CrawlerError::*;
use crate::application::core_types::log::elastic_search::EsDocument;
use crate::application::raven_crawl_task::{
    CrawlTaskError, RavenCrawlTask, TaskFailed, TaskSuccess,
};
use crate::serde_dateformat::yyyy_mm_dd_hh_ii_ss_z;
use chrono::{DateTime, Local};
use rs_es::operations::mapping::Settings;
use serde_derive::Serialize;
use serde_json::Value;
use std::str::FromStr;

#[derive(PartialEq, Debug, Serialize)]
pub struct RavenCrawlTaskMetrics {
    pub total_duration_millis: i64,

    pub name: String,

    #[serde(with = "yyyy_mm_dd_hh_ii_ss_z")]
    pub date: DateTime<Local>,

    pub crawler_duration_millis: Option<i64>,

    pub persist_duration_millis: Option<i64>,

    pub result_code: u16,

    pub result_label: &'static str,

    pub result_detail: String,

    pub task: RavenCrawlTask,

    #[serde(skip_serializing)]
    es_index_name: String,
}

impl RavenCrawlTaskMetrics {
    pub fn new(name: &str, result: &Result<TaskSuccess, TaskFailed>) -> RavenCrawlTaskMetrics {
        let date = Local::now();
        let result_code = match result {
            Ok(_) => 0,
            Err(e) => e.result.err_code(),
        };

        let result_label = match result {
            Ok(_) => "success",
            Err(e) => e.result.err_label(),
        };

        let task = match result {
            Ok(success) => success.task.clone(),
            Err(fail) => fail.task.clone(),
        };

        let crawler_duration_millis: Option<i64> = match result {
            Ok(success) => Some(
                success
                    .result
                    .crawler_result
                    .mills_takes_to_complete_to_request,
            ),
            Err(fail) => match &fail.result {
                CrawlTaskError::CrawlerFailed(crawler_err) => match crawler_err {
                    ClientError(crawler_result) => {
                        Some(crawler_result.mills_takes_to_complete_to_request)
                    }
                    ServerError(crawler_result) => {
                        Some(crawler_result.mills_takes_to_complete_to_request)
                    }
                    TimeoutError { timeout_second, .. } => {
                        Some((timeout_second.clone() as i64) * 1000)
                    }
                    CharsetConversionError { crawler_result, .. } => {
                        Some(crawler_result.mills_takes_to_complete_to_request)
                    }
                    OtherError { .. } => None,
                },
                CrawlTaskError::PersistFailed { crawler_result, .. } => {
                    Some(crawler_result.mills_takes_to_complete_to_request)
                }
            },
        };

        let persist_duration_millis: Option<i64> = match result {
            Ok(success) => Some(
                success
                    .result
                    .crawler_result
                    .mills_takes_to_complete_to_request,
            ),
            Err(fail) => match fail.result {
                CrawlTaskError::CrawlerFailed(_) => None,
                CrawlTaskError::PersistFailed {
                    persist_duration_millis,
                    ..
                } => Some(persist_duration_millis.clone()),
            },
        };

        let total_duration_millis: i64 = match result {
            Ok(success) => success.total_duration_millis,
            Err(fail) => fail.total_duration_millis,
        };

        let result_detail = match result {
            Ok(_) => "success".to_owned(),
            Err(fail) => fail.result.to_string(),
        };

        RavenCrawlTaskMetrics {
            total_duration_millis,
            name: name.to_owned(),
            date,
            crawler_duration_millis,
            persist_duration_millis,
            result_code,
            result_label,
            result_detail,
            task,
            es_index_name: date.format("raven-task-metrics-%Y-%m-%d").to_string(),
        }
    }
}

impl<'a> EsDocument<'a> for RavenCrawlTaskMetrics {
    fn elastic_search_template() -> &'a Value {
        static MAPPING_JSON: &'static str = r#"
            {
                "index_patterns": ["raven-task-metrics-*"],
                               
                "mappings": {
                    "raven-task-metrics": {
                        "properties": {
                            "total_duration": { "type": "integer" },
                            
                            "name": { "type": "keyword" },
                            
                            "date": { 
                                "type": "date",
                                "format": "yyyy-MM-dd HH:mm:ssZZ"
                            },
                            
                            "crawler_duration_millis": { "type": "long" },
                            
                            "persist_duration_millis": { "type": "long" },
                            
                            "result_code": { "type": "integer" },
                            
                            "result_label": { "type": "keyword" },
                            
                            "result_detail": { "type": "text" },
                            
                            "task": {
                                "type": "nested",
                                "properties": {
                                    "request": { "type": "object" },
                                    "persist_method": { "type": "nested" }
                                }
                            }
                        }
                    }
                }    
            }
        "#;

        lazy_static! {
            static ref MAPPING: Value = Value::from_str(MAPPING_JSON).unwrap();
        }

        &*MAPPING
    }

    fn elastic_search_typename() -> &'a str {
        "raven-task-metrics"
    }

    fn elastic_search_settings() -> Option<&'a Settings> {
        None
    }

    fn elastic_search_index_name(&self) -> &str {
        &self.es_index_name
    }
}

#[test]
fn mapping_json_test() {
    /// if MAPPING_JSON is malformed, this test causes panic.
    println!("{}", RavenCrawlTaskMetrics::elastic_search_template());
}
