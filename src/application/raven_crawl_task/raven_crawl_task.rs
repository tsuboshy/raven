use crate::application::raven_crawl_task::task_result::CrawlerTaskResult;
use crate::{
    application::core_types::{
        crawler::{Crawler, CrawlerRequest, CrawlerResult},
        persist::{Persist, PersistError, PersistMethod},
    },
    application::raven_crawl_task::{CrawlTaskError, CrawlTaskSuccess},
};
use chrono::Local;
use serde_derive::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct RavenCrawlTask {
    pub request: CrawlerRequest,
    pub persist_methods: Vec<PersistMethod>,
}

impl RavenCrawlTask {
    pub fn execute_in<App>(
        self,
        app: &App,
    ) -> Result<CrawlerTaskResult<CrawlTaskSuccess>, CrawlerTaskResult<CrawlTaskError>>
    where
        App: Persist + Crawler,
    {
        let start_timestamp = Local::now().timestamp_millis();
        let result = self.execute_task(app);
        let total_duration_millis = Local::now().timestamp_millis() - start_timestamp;

        match result {
            Ok(success) => Ok(CrawlerTaskResult {
                task: self,
                total_duration_millis,
                result: success,
            }),
            Err(err) => Err(CrawlerTaskResult {
                task: self,
                total_duration_millis,
                result: err,
            }),
        }
    }

    fn execute_task<App>(&self, app: &App) -> Result<CrawlTaskSuccess, CrawlTaskError>
    where
        App: Persist + Crawler,
    {
        let crawler_result: CrawlerResult = app.crawl(&self.request)?;

        let persist_start = Local::now().timestamp_millis();
        let mut persist_results = vec![];
        for persist_method in &self.persist_methods {
            let result = app.persist_data(
                persist_method,
                &crawler_result.response_body,
                &crawler_result.response_content_type,
            );
            persist_results.push(result);
        }
        let persist_duration_millis = Local::now().timestamp_millis() - persist_start;

        let all_persist_method_failed = persist_results.iter().all(|result| result.is_err());

        let persist_errors: Vec<PersistError> = persist_results
            .into_iter()
            .flat_map(|result| result.err())
            .collect();

        if all_persist_method_failed {
            Err(CrawlTaskError::PersistFailed {
                crawler_result,
                persist_errors,
                persist_duration_millis,
            })
        } else {
            let result = CrawlTaskSuccess {
                persist_duration_millis,
                crawler_result,
                persist_errors,
            };

            Ok(result)
        }
    }
}
