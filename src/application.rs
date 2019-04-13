use crate::crawl::{CrawlResult, Crawler, CrawlerError, Request};
use crate::input::{RavenConfig, RavenRequest};
use crate::output::output_method::{Output, OutputError, OutputMethod};
use chrono::Local;
use core::borrow::Borrow;
use futures::future::result;
use futures::*;
use futures_cpupool::{CpuFuture, CpuPool};
use std::sync::{Arc, Mutex};

pub trait RavenApplication: Output + Crawler {}

pub fn run_raven_application<App>(config: RavenConfig)
where
    App: RavenApplication,
{
    let thread_size = config.max_threads;
    match config.create_crawler_requests() {
        Ok(requests) => {
            dbg!(crawl_in_parallel::<App>(thread_size, requests));
        }
        Err(err_msg) => error!("failed to create request:{}", dbg!(err_msg)),
    }
}

fn crawl_in_parallel<App>(
    thread_size: u16,
    requests: Vec<Request>,
) -> Vec<Result<RavenResult, RavenError>>
where
    App: RavenApplication,
{
    let mut tasks: Vec<CpuFuture<RavenResult, RavenError>> = Vec::with_capacity(requests.len());
    let thread_pool = CpuPool::new(thread_size.into());

    for request in requests.into_iter() {
        tasks.push(thread_pool.spawn_fn(|| {
            let task_start = Local::now().timestamp_millis();

            let crawler_result: CrawlResult = App::crawl(&request)?;

            let output_start = Local::now().timestamp_millis();
            let mut output_results = vec![];
            for output_method in request.output_methods {
                let result = App::output_crawled_data(
                    &crawler_result.body,
                    crawler_result.content_type.clone(),
                    output_method,
                );
                output_results.push(result);
            }
            let output_duration_millis = Local::now().timestamp_millis() - output_start;

            let all_output_method_failed = output_results.iter().all(|result| result.is_err());

            let output_errors: Vec<OutputError> = output_results
                .into_iter()
                .flat_map(|result| result.err())
                .collect();

            if all_output_method_failed {
                Err(RavenError::OutputFailed(output_errors))
            } else {
                let total_duration_millis = Local::now().timestamp_millis() - task_start;

                let result = RavenResult {
                    total_duration_millis,
                    output_duration_millis,
                    crawler_result,
                    output_errors,
                };

                Ok(result)
            }
        }));
    }

    tasks.into_iter().map(|task| task.wait()).collect()
}

pub struct Prd {}
impl RavenApplication for Prd {}
impl Output for Prd {}
impl Crawler for Prd {}

#[derive(Eq, PartialEq, Debug)]
pub struct RavenResult {
    pub total_duration_millis: i64,
    pub output_duration_millis: i64,
    pub crawler_result: CrawlResult,
    pub output_errors: Vec<OutputError>,
}

#[derive(Eq, PartialEq, Debug)]
pub enum RavenError {
    CrawlerFailed(CrawlerError),

    OutputFailed(Vec<OutputError>),
}

impl From<CrawlerError> for RavenError {
    fn from(e: CrawlerError) -> Self {
        RavenError::CrawlerFailed(e)
    }
}

impl From<Vec<OutputError>> for RavenError {
    fn from(e: Vec<OutputError>) -> Self {
        RavenError::OutputFailed(e)
    }
}
