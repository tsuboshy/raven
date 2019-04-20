use super::boundary::CommandLineRaven;
use crate::application::{
    command_runner::{config::config::RavenConfig, config::log::LogConfig},
    core_types::{
        crawler::Crawler,
        logger::{LogLevel, Logger},
        notify_method::{Notify, NotifyError},
        persist::Persist,
    },
};

use crate::application::command_runner::config::config::HasConfig;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::fmt::Debug;

pub struct Prd {
    config: RavenConfig,
}

impl Prd {
    pub fn init(config: RavenConfig) -> Prd {
        let prd = Prd { config };
        let log_config = log_config(&prd.config.log);
        log4rs::init_config(log_config).unwrap();
        prd
    }
}

fn log_config(log_config: &LogConfig) -> Config {
    let file_append = FileAppender::builder()
        .append(true)
        .encoder(Box::new(PatternEncoder::new("{d} - [{l}]\t{m}{n}")))
        .build(&log_config.file_path)
        .unwrap();

    Config::builder()
        .appender(Appender::builder().build("file", Box::new(file_append)))
        .build(
            Root::builder()
                .appender("file")
                .build(to_log_level(&log_config.level)),
        )
        .unwrap()
}

fn to_log_level(log_level: &LogLevel) -> LevelFilter {
    match log_level {
        LogLevel::Trace => LevelFilter::Trace,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Error => LevelFilter::Error,
    }
}

impl CommandLineRaven for Prd {}

impl HasConfig for Prd {
    fn get_config(&self) -> &RavenConfig {
        &self.config
    }
}

impl Persist for Prd {}

impl Logger for Prd {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Trace => trace!("{}", message),
            LogLevel::Debug => debug!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Error => error!("{}", message),
        }
    }

    fn log_trace<T: Debug>(&self, label: &str, object: &T) {
        trace!("{}: {:?}", label, object);
    }

    fn log_debug<T: Debug>(&self, label: &str, object: &T) {
        debug!("{}: {:?}", label, object);
    }
}

impl Notify for Prd {
    fn notify(&self, level: LogLevel, message: &str) -> Result<(), NotifyError> {
        Ok(())
    }
}

impl Crawler for Prd {}
