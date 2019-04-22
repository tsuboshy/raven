use super::{
    log::LogConfig,
    notify_method::NotifyMethod,
    parser::{product_list, try_expand_numeric_list, TemplateBuilder},
    raven_request::RavenRequest,
};
use crate::application::{
    core_types::{
        crawler::request::{CrawlerRequest, Method},
        persist::PersistMethod,
    },
    raven_crawl_task::*,
};
use chrono::{DateTime, Local};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Deserialize)]
pub struct RavenConfig {
    pub name: String,

    pub request: RavenRequest,

    #[serde(default)]
    pub notify: Vec<NotifyMethod>,

    pub output: Vec<PersistMethod>,

    #[serde(default = "default_max_threads")]
    pub max_threads: u16,

    pub log: LogConfig,
}

fn default_max_threads() -> u16 {
    1
}

pub trait HasConfig {
    fn get_config(&self) -> &RavenConfig;
}

impl RavenConfig {
    pub fn create_crawler_tasks(&self) -> Result<Vec<RavenCrawlTask>, String> {
        // template builders
        let now = Local::now();

        let time_formatted_url = now.format(&self.request.url).to_string();
        let url_template_builder = TemplateBuilder::new(&time_formatted_url);

        let mut persist_method_template_builders: Vec<(&PersistMethod, TemplateBuilder)> =
            Vec::with_capacity(self.output.len());
        for persist_method in &self.output {
            let time_formatted_file_name = now.format(persist_method.get_file_name()).to_string();
            persist_method_template_builders.push((
                persist_method,
                TemplateBuilder::new(&time_formatted_file_name),
            ));
        }

        // var map
        let mut var_maps_list = self
            .request
            .vars
            .iter()
            .flat_map(|var_map| parse_key_value_map(var_map))
            .collect::<Vec<HashMap<String, String>>>();

        if var_maps_list.is_empty() {
            var_maps_list.push(HashMap::new());
        }

        // param map
        let mut param_map_list = self
            .request
            .params
            .iter()
            .flat_map(|params| parse_key_value_map(params))
            .collect::<Vec<HashMap<String, String>>>();

        if param_map_list.is_empty() {
            param_map_list.push(HashMap::new());
        }

        let mut request_list: Vec<RavenCrawlTask> =
            Vec::with_capacity(var_maps_list.len() * param_map_list.len());

        for (var_map, param_map) in product_list(&var_maps_list, &param_map_list) {
            request_list.push(self.create_crawler_request(
                var_map.clone(),
                param_map.clone(),
                &url_template_builder,
                &persist_method_template_builders,
            )?);
        }

        Ok(request_list)
    }

    fn create_crawler_request(
        &self,
        var_map: HashMap<String, String>,
        param_map: HashMap<String, String>,
        url_builder: &TemplateBuilder,
        persist_method_with_template_builders: &[(&PersistMethod, TemplateBuilder)],
    ) -> Result<RavenCrawlTask, String> {
        let mut all_val_map: HashMap<&str, &str> = HashMap::new();
        copy_ref_to_other_map!(var_map, all_val_map);
        copy_ref_to_other_map!(param_map, all_val_map);

        let url = url_builder.build_string(&all_val_map)?;
        let mut persist_method_list: Vec<PersistMethod> = Vec::new();

        for (persist_method, file_path_builder) in persist_method_with_template_builders.iter() {
            let embedded_val_file_name = file_path_builder.build_string(&all_val_map)?;
            let mut cloned_method = (*persist_method).clone();
            cloned_method.update_file_path(embedded_val_file_name);
            persist_method_list.push(cloned_method);
        }

        let (query_map, body_map) = match self.request.method {
            Method::Get => (param_map, HashMap::new()),
            Method::Post => (HashMap::new(), param_map),
        };

        let request = CrawlerRequest {
            url,
            method: self.request.method.clone(),
            header: self.request.headers.clone(),
            timeout: self.request.timeout_in_seconds,
            max_retry: self.request.max_retry,
            encoding_setting: self.request.encoding.clone(),
            query_params: query_map,
            body_params: body_map,
        };

        let task = RavenCrawlTask {
            request,
            persist_methods: persist_method_list,
        };

        Ok(task)
    }
}

#[test]
fn create_request_from_config_test() {
    use crate::application::core_types::logger::LogLevel;
    let var: HashMap<String, Vec<String>> = hashmap![
        "id".to_owned() => vec!["1".to_owned(), "2".to_owned()]
    ];

    let param_1: HashMap<String, Vec<String>> = hashmap![
        "offset".to_owned() => vec!["0".to_owned()],
        "limit".to_owned() => vec!["100".to_owned()]
    ];

    let param_2: HashMap<String, Vec<String>> = hashmap![
        "offset".to_owned() => vec!["100".to_owned(), "300".to_owned()],
        "limit".to_owned() => vec!["200".to_owned()]
    ];

    let raven_config = RavenConfig {
        name: "test_config".to_owned(),
        request: RavenRequest {
            url: "http://test.com/{{id}}".to_owned(),
            method: Method::Get,
            headers: HashMap::new(),
            vars: vec![var],
            encoding: None,
            timeout_in_seconds: 5,
            max_retry: 1,
            params: vec![param_1, param_2],
        },
        notify: Vec::new(),
        output: vec![PersistMethod::AmazonS3 {
            region: "ap-northeast-1".to_owned(),
            bucket_name: "application".to_owned(),
            object_key: "test/%Y%m%d/{{id}}_{{offset}}_{{limit}}.html".to_owned(),
        }],
        max_threads: 1,
        log: LogConfig {
            file_path: "/var/tmp/log".to_owned(),
            level: LogLevel::Debug,
        },
    };

    let expected_url = vec![
        "http://test.com/1".to_owned(),
        "http://test.com/2".to_owned(),
    ];

    let now_y_m_d = Local::now().format("%Y%m%d").to_string();
    let result: Result<Vec<RavenCrawlTask>, String> = raven_config.create_crawler_tasks();

    let expected_object_keys = vec![
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "1", "0", "100"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "2", "0", "100"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "1", "100", "200"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "1", "300", "200"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "2", "100", "200"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "2", "300", "200"),
    ];

    assert_eq!(result.is_ok(), true);
    let tasks: Vec<RavenCrawlTask> = result.unwrap();
    assert_eq!(tasks.len(), 6);
    for task in tasks.iter() {
        assert_eq!(task.request.method, Method::Get);
        assert_eq!(task.request.header.len(), 0);
        assert_eq!(task.request.encoding_setting, None);
        assert_eq!(task.request.timeout, 5);
        assert_eq!(task.request.max_retry, 1);
        assert_eq!(task.request.body_params.len(), 0);
        assert_eq!(expected_url.contains(&task.request.url), true);
        let file_name = match &task.persist_methods[0] {
            PersistMethod::LocalFile { .. } => panic!("must not be LocalFile."),
            PersistMethod::AmazonS3 { object_key, .. } => object_key,
        };
        assert_eq!(expected_object_keys.contains(&file_name), true);
    }
}

fn parse_key_value_map(map: &HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    if map.is_empty() {
        return vec![HashMap::new()];
    }

    let now: DateTime<Local> = Local::now();

    // from Vec<HashMap<String, Vec<String>>>: [("name", ["tsuboshy", "sato"]), ("event_date", ["2019-04-1[1..2]")]]
    // to   Vec<Vec<HashMap<String, String>>>: [[("name", "tsuboshy"), ("name", "sato")], [("event_date", ""2019-04-11"), ("event_date", "2019-04-12")]]
    let mut single_map_lists: Vec<Vec<HashMap<&str, String>>> = vec![];
    for (key, values) in map.into_iter() {
        let single_maps = values
            .iter()
            .map(|val| now.format(val).to_string())
            .flat_map(|val| try_expand_numeric_list(&val))
            .map(|val| {
                let mut single_map = HashMap::new();
                single_map.insert(key.as_str(), val);
                single_map
            })
            .collect::<Vec<HashMap<&str, String>>>();
        single_map_lists.push(single_maps);
    }

    let empty_map: HashMap<&str, &str> = HashMap::new();

    let list_of_ref_maps: Vec<HashMap<&str, &str>> =
        single_map_lists
            .iter()
            .fold(vec![empty_map], |until_now_maps, next_maps| {
                let mut new_result_list: Vec<HashMap<&str, &str>> = Vec::new();
                for until_now_map in &until_now_maps {
                    for next_map in next_maps {
                        let mut result_map: HashMap<&str, &str> = HashMap::new();
                        copy_ref_to_other_map!(next_map, result_map);
                        copy_ref_to_other_map!(until_now_map, result_map);
                        new_result_list.push(result_map);
                    }
                }
                new_result_list
            });

    let list_of_owned_maps = list_of_ref_maps
        .into_iter()
        .map(|result_map| {
            result_map
                .into_iter()
                .map(|(key, val): (&str, &str)| (key.to_owned(), val.to_owned()))
                .collect::<HashMap<String, String>>()
        })
        .collect();

    list_of_owned_maps
}

#[test]
fn parse_key_value_map_test() {
    let now_y_m_d = Local::now().format("%Y-%m-%d");
    let test_map: HashMap<String, Vec<String>> = hashmap![
        "name".to_owned() => vec!["tsuboshy".to_owned(), "sato".to_owned()],
        "event_date".to_owned() =>  vec!["2019-04-1[1..2]".to_owned()],
        "test_date".to_owned() => vec!["%Y-%m-%d".to_owned()]
    ];

    let results: Vec<HashMap<String, String>> = dbg!(parse_key_value_map(&test_map));

    let expected: Vec<HashMap<String, String>> = vec![
        hashmap![
            "name".to_owned() => "tsuboshy".to_owned(),
            "event_date".to_owned() => "2019-04-11".to_owned(),
            "test_date".to_owned() => now_y_m_d.to_string()
        ],
        hashmap![
            "name".to_owned() => "tsuboshy".to_owned(),
            "event_date".to_owned() => "2019-04-12".to_owned(),
            "test_date".to_owned() => now_y_m_d.to_string()
        ],
        hashmap![
            "name".to_owned() => "sato".to_owned(),
            "event_date".to_owned() => "2019-04-11".to_owned(),
            "test_date".to_owned() => now_y_m_d.to_string()
        ],
        hashmap![
            "name".to_owned() => "sato".to_owned(),
            "event_date".to_owned() => "2019-04-12".to_owned(),
            "test_date".to_owned() => now_y_m_d.to_string()
        ],
    ];
    assert_eq!(expected.len(), results.len());
    for result_map in results {
        let result_tuples = result_map.iter().collect::<Vec<(&String, &String)>>();
        let mut matched = false;
        'check_whether_exists_same_one_in_expected_maps: for expected_map in &expected {
            let expected_tuples = expected_map.iter().collect::<Vec<(&String, &String)>>();
            for result_tuple in &result_tuples {
                if expected_tuples.contains(&result_tuple) {
                    matched = true;
                    break 'check_whether_exists_same_one_in_expected_maps;
                }
            }
        }
        if !matched {
            panic!("result_map is not expected!: {:?}", result_map);
        }
    }
}

#[test]
fn parse_key_value_map_and_template_parser_test() {
    let now_y_m_d = Local::now().format("%Y-%m-%d").to_string();

    let builder = TemplateBuilder::new("https://application/{{a}}/{{b}}/{{c}}");

    let map = hashmap![
        "a".to_owned() => vec!["a1".to_owned(), "a2".to_owned()],
        "b".to_owned() => vec!["b[1..2]".to_owned()],
        "c".to_owned() => vec!["c1-%Y-%m-%d".to_owned()]
    ];

    let expected = vec![
        ["https://application/a1/b1/c1-", &now_y_m_d].concat(),
        ["https://application/a1/b2/c1-", &now_y_m_d].concat(),
        ["https://application/a2/b1/c1-", &now_y_m_d].concat(),
        ["https://application/a2/b2/c1-", &now_y_m_d].concat(),
    ];

    let parsed_map_list = parse_key_value_map(&map);
    assert_eq!(parsed_map_list.len(), expected.len());

    for parsed in parsed_map_list {
        let embedded = builder.build_string(&parsed).unwrap();
        assert!(expected.contains(&embedded));
    }
}
