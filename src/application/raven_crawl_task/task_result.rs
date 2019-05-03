use crate::application::core_types::crawler::metrics::CrawlerMetrics;
use crate::application::core_types::{crawler::CrawlerResult, persist::PersistError};
use crate::application::raven_crawl_task::{CrawlTaskError, RavenCrawlTask};

#[derive(PartialEq, Debug)]
pub struct CrawlTaskSuccess {
    pub persist_duration_millis: i64,
    pub crawler_result: CrawlerResult,
    pub persist_errors: Vec<PersistError>,
}

#[derive(PartialEq, Debug)]
pub struct CrawlerTaskResult<T> {
    pub task: RavenCrawlTask,
    pub total_duration_millis: i64,
    pub result: T,
}

pub type TaskSuccess = CrawlerTaskResult<CrawlTaskSuccess>;
pub type TaskFailed = CrawlerTaskResult<CrawlTaskError>;

pub fn to_crawler_metrics(
    crawler_name: &str,
    result: Result<TaskSuccess, TaskFailed>,
) -> CrawlerMetrics {
    match result {
        Ok(success) => CrawlerMetrics::new(
            crawler_name,
            success.task.request,
            &Ok(success.result.crawler_result),
        ),
        Err(fail) => match fail.result {
            CrawlTaskError::CrawlerFailed(crawler_fail) => {
                CrawlerMetrics::new(crawler_name, fail.task.request, &Err(crawler_fail))
            }
            CrawlTaskError::PersistFailed { crawler_result, .. } => {
                CrawlerMetrics::new(crawler_name, fail.task.request, &Ok(crawler_result))
            }
        },
    }
}
