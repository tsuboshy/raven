use super::boundary::CommandLineRaven;
use crate::application::{
    command_runner::{
        config::config::RavenConfig, config::log::LogConfig, config::notify_method::NotifyMethod,
    },
    core_types::{
        crawler::Crawler,
        logger::LogLevel,
        notify::{Notify, NotifyError},
        persist::Persist,
    },
};

use crate::application::command_runner::config::config::HasConfig;
use crate::application::command_runner::config::notify_method::send_to_slack;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

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

impl Notify for Prd {
    fn notify(
        &self,
        notify_level: LogLevel,
        label: &str,
        message: &str,
    ) -> Result<(), NotifyError> {
        let mut err_msgs: Vec<String> = vec![];
        for notify_method in &self.config.notify {
            match notify_method {
                NotifyMethod::Slack {
                    url,
                    channel,
                    mention,
                    level,
                } if level == &notify_level => {
                    let username = format!("raven - {}", &self.config.name);
                    let send_result = send_to_slack(
                        url,
                        channel,
                        mention.as_ref(),
                        &username,
                        level,
                        label,
                        message,
                    );
                    if let Err(err) = send_result {
                        err_msgs.push(err.0);
                    }
                }

                _ => (),
            }
        }

        if err_msgs.is_empty() {
            Ok(())
        } else {
            Err(NotifyError(err_msgs.join(", ")))
        }
    }
}

impl Crawler for Prd {}
