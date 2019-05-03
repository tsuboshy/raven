use crate::application::{
    command_runner::config::config::HasConfig,
    core_types::{crawler::Crawler, notify_method::Notify, persist::Persist},
    raven_crawl_task::{RavenCrawlTask, TaskSuccess},
};
use futures::future::Future;
use futures_cpupool::{CpuFuture, CpuPool};
use std::sync::Arc;

use crate::application::core_types::crawler::metrics::CrawlerMetrics;
use crate::application::core_types::log::elastic_search::{BulkInsertToEs, EsDocument};
use crate::application::core_types::logger::write_error_log_if_err;
use crate::application::raven_crawl_task::task_result::to_crawler_metrics;
use crate::application::raven_crawl_task::{RavenCrawlTaskMetrics, TaskFailed};

pub trait CommandLineRaven: HasConfig + Crawler + Persist + Notify + BulkInsertToEs {}

pub fn run_raven_application<App>(app: App)
where
    App: CommandLineRaven + Sync + Send + 'static,
{
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
            let task_metrics: Vec<RavenCrawlTaskMetrics> = task_result
                .iter()
                .map(|result| RavenCrawlTaskMetrics::new(&app_arc.get_config().name, result))
                .collect();

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
            let err_msg = format!("failed to create request: {}", err);
            error!("{}", err_msg);
            let _ = app.notify_error(&err_msg);
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
        future_list.push(thread_pool.spawn_fn(move || task.execute_in(&*cloned_app_arc)));
    }

    let task_results = future_list
        .into_iter()
        .map(|future| future.wait())
        .collect();

    info!("complete all crawler tasks");

    task_results
}
