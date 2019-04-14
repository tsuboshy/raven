use super::core_types::{
    crawler::{Crawler, CrawlerError, CrawlerRequest, CrawlerResult},
    persist::{Persist, PersistError, PersistMethod},
};
use chrono::Local;

#[derive(Debug)]
pub struct RavenCrawlTask {
    pub request: CrawlerRequest,
    pub persist_methods: Vec<PersistMethod>,
}

impl RavenCrawlTask {
    pub fn execute_in<App>(self, app: &App) -> Result<CrawlTaskResult, CrawlTaskError>
    where
        App: Persist + Crawler,
    {
        let task_start = Local::now().timestamp_millis();

        let crawler_result: CrawlerResult = app.crawl(&self.request)?;

        let persist_start = Local::now().timestamp_millis();
        let mut persist_results = vec![];
        for persist_method in self.persist_methods {
            let result = app.persist_data(
                persist_method,
                &crawler_result.response_body,
                &crawler_result.response_content_type,
            );
            persist_results.push(result);
        }
        let output_duration_millis = Local::now().timestamp_millis() - persist_start;

        let all_output_method_failed = persist_results.iter().all(|result| result.is_err());

        let output_errors: Vec<PersistError> = persist_results
            .into_iter()
            .flat_map(|result| result.err())
            .collect();

        if all_output_method_failed {
            Err(CrawlTaskError::OutputFailed(output_errors))
        } else {
            let total_duration_millis = Local::now().timestamp_millis() - task_start;

            let result = CrawlTaskResult {
                total_duration_millis,
                output_duration_millis,
                crawler_result,
                output_errors,
            };

            Ok(result)
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct CrawlTaskResult {
    pub total_duration_millis: i64,
    pub output_duration_millis: i64,
    pub crawler_result: CrawlerResult,
    pub output_errors: Vec<PersistError>,
}

#[derive(Eq, PartialEq, Debug)]
pub enum CrawlTaskError {
    CrawlerFailed(CrawlerError),

    OutputFailed(Vec<PersistError>),
}

impl From<CrawlerError> for CrawlTaskError {
    fn from(e: CrawlerError) -> Self {
        CrawlTaskError::CrawlerFailed(e)
    }
}

impl From<Vec<PersistError>> for CrawlTaskError {
    fn from(e: Vec<PersistError>) -> Self {
        CrawlTaskError::OutputFailed(e)
    }
}
