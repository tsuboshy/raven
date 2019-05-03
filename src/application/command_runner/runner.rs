use chrono::{DateTime, Local};
use futures::future::Future;
use futures_cpupool::{CpuFuture, CpuPool};
use hostname::get_hostname;
use std::sync::Arc;

use crate::application::{
    command_runner::config::config::HasConfig,
    core_types::{
        crawler::metrics::CrawlerMetrics,
        crawler::Crawler,
        log::elastic_search::{BulkInsertToEs, EsDocument},
        logger::write_error_log_if_err,
        notify::Notify,
        persist::Persist,
    },
    raven_crawl_task::{
        task_result::to_crawler_metrics, RavenCrawlTask, RavenCrawlTaskMetrics, TaskFailed,
        TaskSuccess,
    },
};

pub trait CommandLineRaven: HasConfig + Crawler + Persist + Notify + BulkInsertToEs {}

pub fn run_raven_application<App>(app: App)
where
    App: CommandLineRaven + Sync + Send + 'static,
{
    let start_time = Local::now();
    info!("raven application start: {}", app.get_config().name);
    debug!("raven config: {:?}", app.get_config());

    let _ = write_error_log_if_err(
        "failed to create elasticsearch crawler-metrics index template",
        app.create_index_template::<CrawlerMetrics>(CrawlerMetrics::elastic_search_typename()),
    );

    let _ = write_error_log_if_err(
        "failed to create elasticsearch task-metrics index template",
        app.create_index_template::<RavenCrawlTaskMetrics>(
            RavenCrawlTaskMetrics::elastic_search_typename(),
        ),
    );

    let thread_size = app.get_config().max_threads;
    match app.get_config().create_crawler_tasks() {
        Ok(tasks) => {
            let app_arc = Arc::new(app);
            let task_result: Vec<Result<TaskSuccess, TaskFailed>> =
                crawl_in_parallel(app_arc.clone(), thread_size, tasks);

            let total_duration = Local::now().timestamp_millis() - start_time.timestamp_millis();

            let task_metrics: Vec<RavenCrawlTaskMetrics> = task_result
                .iter()
                .map(|result| RavenCrawlTaskMetrics::new(&app_arc.get_config().name, result))
                .collect();

            notify_result(app_arc.as_ref(), &start_time, total_duration, &task_result);

            let _ = write_error_log_if_err(
                "failed to insert task metrics to es",
                app_arc.bulk_insert(&task_metrics),
            );

            let crawler_metrics: Vec<CrawlerMetrics> = task_result
                .into_iter()
                .map(|result| to_crawler_metrics(&app_arc.get_config().name, result))
                .collect();

            let _ = write_error_log_if_err(
                "failed to insert crawler metrics to es",
                app_arc.bulk_insert(&crawler_metrics),
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
) -> Vec<Result<TaskSuccess, TaskFailed>>
where
    App: CommandLineRaven + Sync + Send + 'static,
{
    info!("num of crawler tasks: {}", tasks.len());
    info!("thread size: {}", thread_size);
    debug!("tasks detail: {:?}", &tasks);

    let mut future_list: Vec<CpuFuture<TaskSuccess, TaskFailed>> = Vec::with_capacity(tasks.len());
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
    results: &[Result<TaskSuccess, TaskFailed>],
) where
    App: CommandLineRaven,
{
    let hostname = get_hostname().unwrap_or("unknown host".to_owned());

    let mut total_failure_num = 0;
    let mut persist_errors_num = 0;

    for result in results {
        match result {
            Ok(crawler_result) => {
                persist_errors_num += crawler_result.result.persist_errors.len();
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
        failure task num:    {} 
        failure request num: {} 
        output failure num:  {} ",
        &app.get_config().name,
        hostname,
        start_time.format("%F %T"),
        total_duration / 1000,
        results.len(),
        total_failure_num,
        persist_errors_num,
    );

    let _ = app.notify_info("raven command is completed.", &notify_message);
}
