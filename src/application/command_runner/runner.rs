use crate::application::{
    command_runner::{config::config::HasConfig, config::config::RavenConfig},
    core_types::{crawler::Crawler, logger::Logger, notify_method::Notify, persist::Persist},
    raven_crawl_task::{CrawlTaskError, CrawlTaskResult, RavenCrawlTask},
};
use futures::future::Future;
use futures_cpupool::{CpuFuture, CpuPool};
use std::sync::Arc;

pub trait CommandLineRaven: HasConfig + Crawler + Persist + Notify + Logger {}

pub fn run_raven_application<App>(app: App)
where
    App: CommandLineRaven + Sync + Send + 'static,
{
    app.log_info(&format!(
        "raven application start: {}",
        app.get_config().name
    ));
    app.log_debug("config", app.get_config());
    let thread_size = app.get_config().max_threads;
    match app.get_config().create_crawler_tasks() {
        Ok(tasks) => {
            crawl_in_parallel(Arc::new(app), thread_size, tasks);
        }
        Err(err) => {
            let err_msg = format!("failed to create request: {}", err);
            app.log_error(&err_msg);
            app.log_error_if_err(app.notify_error(&err_msg));
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
    app_arc.log_info(&format!("num of crawler tasks: {}", tasks.len()));
    app_arc.log_debug("tasks detail", &tasks);

    let mut future_list: Vec<CpuFuture<CrawlTaskResult, CrawlTaskError>> =
        Vec::with_capacity(tasks.len());
    let thread_pool = CpuPool::new(thread_size.into());

    for task in tasks.into_iter() {
        let cloned_app_arc = app_arc.clone();
        future_list.push(thread_pool.spawn_fn(move || task.execute_in(&*cloned_app_arc)));
    }

    let task_results = future_list
        .into_iter()
        .map(|future| future.wait())
        .collect();

    app_arc.log_info("complete all crawler tasks");

    task_results
}
