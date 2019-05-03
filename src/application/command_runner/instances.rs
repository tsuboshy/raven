use super::boundary::CommandLineRaven;
use crate::application::{
    command_runner::{config::config::RavenConfig, config::log::LogConfig},
    core_types::{
        crawler::Crawler,
        logger::LogLevel,
        notify::{Notify, NotifyError},
        persist::Persist,
    },
};

use crate::application::command_runner::config::config::HasConfig;
use crate::application::command_runner::config::notify_method::{send_to_slack, NotifyMethod};
use crate::application::core_types::log::elastic_search::{BulkInsertToEs, EsDocument};
use crate::es_api::create_es_index_if_not_exists;
use crate::macros::HashMap;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use rs_es::error::EsError;
use rs_es::operations::bulk::Action;
use rs_es::Client;
use std::io::{Error, ErrorKind};
use uuid::Uuid;

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
        .build(&log_config.file.path)
        .unwrap();

    Config::builder()
        .appender(Appender::builder().build("file", Box::new(file_append)))
        .build(
            Root::builder()
                .appender("file")
                .build(to_log_level(&log_config.file.level)),
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

impl BulkInsertToEs for Prd {
    fn bulk_insert<'a, T>(&self, documents: &[T]) -> Result<(), EsError>
    where
        T: EsDocument<'a>,
    {
        if let Some(es_config) = &self.config.log.elasticsearch {
            let mut actions_by_index_name: HashMap<&str, Vec<Action<&T>>> = HashMap::new();

            for doc in documents {
                if !actions_by_index_name.contains_key(doc.elastic_search_index_name()) {
                    actions_by_index_name.insert(doc.elastic_search_index_name(), vec![]);
                }

                let same_index_docs: &mut Vec<Action<&T>> = actions_by_index_name
                    .get_mut(doc.elastic_search_index_name())
                    .expect("unreachable code! @ impl BulkInsertToEs for Prd");

                let action = Action::create(doc)
                    .with_id(Uuid::new_v4().to_string())
                    .with_doc_type(T::elastic_search_typename());

                same_index_docs.push(action);
            }

            let mut client = Client::init(&es_config.endpoint).unwrap();

            // bulk by index name
            for (index_name, actions) in actions_by_index_name {
                client.bulk(&actions).with_index(index_name).send()?;
            }

            Ok(())
        } else {
            Ok(())
        }
    }

    fn create_index_template<'a, T>(&self, name: &str) -> Result<(), Error>
    where
        T: EsDocument<'a>,
    {
        if let Some(es_config) = &self.config.log.elasticsearch {
            let template_json = T::elastic_search_template();
            let result = create_es_index_if_not_exists(&es_config.endpoint, name, template_json);

            if let Err(err) = result {
                error!("{}", err);
                Err(Error::from(ErrorKind::Other))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

#[test]
fn try_to_create_index() {
    use crate::application::core_types::crawler::metrics::CrawlerMetrics;
    let _ = dbg!(create_es_index_if_not_exists(
        "http://localhost:9200",
        "test",
        CrawlerMetrics::elastic_search_template()
    ));
}
