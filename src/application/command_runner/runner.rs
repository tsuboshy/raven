use crate::application::{
    command_runner::config::config::HasConfig,
    core_types::{crawler::Crawler, notify::Notify, persist::Persist},
    raven_crawl_task::{CrawlTaskError, CrawlTaskResult, RavenCrawlTask},
};
use chrono::{DateTime, Local};
use futures::future::Future;
use futures_cpupool::{CpuFuture, CpuPool};
use hostname::get_hostname;
use std::sync::Arc;

pub trait CommandLineRaven: HasConfig + Crawler + Persist + Notify {}

pub struct CommandLineResult {
    pub crawler_name: String,
    pub hostname: String,
    pub total_duration: i64,
    pub total_request_num: u32,
    pub failure_request_num: u32,
    pub output_failure_num: u32,
}

pub fn run_raven_application<App>(app: App)
where
    App: CommandLineRaven + Sync + Send + 'static,
{
    info!("raven application start: {}", app.get_config().name);
    debug!("raven config: {:?}", app.get_config());
    let thread_size = app.get_config().max_threads;
    match app.get_config().create_crawler_tasks() {
        Ok(tasks) => {
            let app_arc = Arc::new(app);
            let start_time = Local::now();
            let crawler_result = crawl_in_parallel(app_arc.clone(), thread_size, tasks);
            let total_duration = Local::now().timestamp_millis() - start_time.timestamp_millis();
            notify_result(
                app_arc.as_ref(),
                &start_time,
                total_duration,
                &crawler_result,
            );
        }
        Err(err) => {
            error!("failed to create request: {}", err);
            let _ = app.notify_error("failed to create request", &err);
        }
    }
}

fn crawl_in_parallel<App>(
    app_arc: Arc<App>,
    thread_size: u16,
    tasks: Vec<RavenCrawlTask>,
) -> Vec<Result<CrawlTaskResult, CrawlTaskError>>
where
    App: CommandLineRaven + Sync + Send + 'static,
{
    info!("num of crawler tasks: {}", tasks.len());
    info!("thread size: {}", thread_size);
    debug!("tasks detail: {:?}", &tasks);

    let mut future_list: Vec<CpuFuture<CrawlTaskResult, CrawlTaskError>> =
        Vec::with_capacity(tasks.len());
    let thread_pool = CpuPool::new(thread_size.into());

    for task in tasks.into_iter() {
        let cloned_app_arc = app_arc.clone();
        future_list.push(thread_pool.spawn_fn(move || task.execute_in(cloned_app_arc.as_ref())));
    }

    let task_results = future_list
        .into_iter()
        .map(|future| future.wait())
        .collect();

    info!("complete all crawler tasks");

    task_results
}

fn notify_result<App>(
    app: &App,
    start_time: &DateTime<Local>,
    total_duration: i64,
    results: &[Result<CrawlTaskResult, CrawlTaskError>],
) where
    App: CommandLineRaven,
{
    let hostname = get_hostname().unwrap_or("unknown host".to_owned());

    let mut total_failure_num = 0;
    let mut output_failure_num = 0;

    for result in results {
        match result {
            Ok(crawler_result) => {
                output_failure_num += crawler_result.output_errors.len();
            }
            Err(_) => {
                total_failure_num += 1;
            }
        }
    }
    let notify_message: String = format!(
        "
        crawler name:        {} 
        crawler hostname:    {} 
        start datetime:      {} 
        total duration:      {} seconds
        total request num:   {} 
        failure request num: {} 
        output failure num:  {} ",
        &app.get_config().name,
        hostname,
        start_time.format("%F %T"),
        total_duration / 1000,
        results.len(),
        total_failure_num,
        output_failure_num,
    );

    let _ = app.notify_info("raven command is completed.", &notify_message);
}
