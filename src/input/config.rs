use chrono::{DateTime, Local};
use serde_derive::Deserialize;
use std::collections::HashMap;

use super::raven_template_parser::{product_list, try_expand_list, TemplateBuilder};
use super::request::RavenRequest;

use crate::{
    crawl::{Method, Request},
    logger::log_level::LogLevel,
    notify::Notify,
    output::*,
};

#[derive(Debug, PartialEq, Deserialize)]
pub struct RavenConfig {
    pub name: String,

    pub request: RavenRequest,

    #[serde(default)]
    pub notify: Vec<Notify>,

    pub output: Vec<OutputMethod>,

    #[serde(default = "default_max_threads")]
    pub max_threads: u16,

    pub log: LogConfig,
}

fn default_max_threads() -> u16 {
    1
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LogConfig {
    pub file_path: String,
    pub level: LogLevel,
}

impl RavenConfig {
    pub fn create_crawler_requests(&self) -> Result<Vec<Request>, String> {
        let now = Local::now();

        let time_formatted_url = now.format(&self.request.url).to_string();
        let url_template_builder = TemplateBuilder::new(&time_formatted_url);

        let mut output_method_template_builders: Vec<(&OutputMethod, TemplateBuilder)> =
            Vec::with_capacity(self.output.len());
        for output_method in &self.output {
            let file_name = match output_method {
                OutputMethod::LocalFile { file_path } => file_path,
                OutputMethod::AmazonS3 { object_key, .. } => object_key,
            };

            let time_formatted_file_name = now.format(file_name).to_string();
            output_method_template_builders.push((
                output_method,
                TemplateBuilder::new(&time_formatted_file_name),
            ));
        }

        let var_maps_list = self
            .request
            .vars
            .iter()
            .flat_map(|var_map| parse_key_value_map(var_map))
            .collect::<Vec<HashMap<String, String>>>();

        let param_map_list = self
            .request
            .params
            .iter()
            .flat_map(|params| parse_key_value_map(params))
            .collect::<Vec<HashMap<String, String>>>();

        let mut request_list: Vec<Request> =
            Vec::with_capacity(var_maps_list.len() * param_map_list.len());

        for (var_map, param_map) in product_list(&var_maps_list, &param_map_list) {
            request_list.push(self.create_crawler_request(
                var_map,
                param_map,
                &url_template_builder,
                &output_method_template_builders,
            )?);
        }

        Ok(request_list)
    }

    fn create_crawler_request(
        &self,
        var_map: HashMap<String, String>,
        param_map: HashMap<String, String>,
        url_builder: &TemplateBuilder,
        output_method_with_template_builders: &Vec<(&OutputMethod, TemplateBuilder)>,
    ) -> Result<Request, String> {
        let mut all_val_map: HashMap<&str, &str> = HashMap::new();
        copy_ref_to_other_map!(var_map, all_val_map);
        copy_ref_to_other_map!(param_map, all_val_map);

        let url = url_builder.build_string(&all_val_map)?;
        let mut output_method_list: Vec<OutputMethod> = Vec::new();

        for (output_method, file_path_builder) in output_method_with_template_builders.iter() {
            let embedded_val_file_name = file_path_builder.build_string(&all_val_map)?;
            let mut cloned_method = (*output_method).clone();
            cloned_method.update_file_path(embedded_val_file_name);
            output_method_list.push(cloned_method);
        }

        let (query_map, body_map) = match self.request.method {
            Method::Get => (param_map, HashMap::new()),
            Method::Post => (HashMap::new(), param_map),
        };

        let request = Request {
            url,
            method: self.request.method.clone(),
            header: self.request.headers.clone(),
            ouput_methods: output_method_list,
            input_charset: self.request.input_charset.to_owned(),
            output_charset: self.request.output_charset.to_owned(),
            timeout: self.request.timeout_in_seconds,
            max_retry: self.request.max_retry,
            val_map: var_map,
            query_params: query_map,
            body_params: body_map,
        };

        Ok(request)
    }
}

#[test]
fn create_request_from_config_test() {
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
            input_charset: "UTF-8".to_owned(),
            output_charset: "UTF-8".to_owned(),
            timeout_in_seconds: 5,
            max_retry: 1,
            params: vec![param_1, param_2],
        },
        notify: Vec::new(),
        output: vec![OutputMethod::AmazonS3 {
            region: "ap-northeast-1".to_owned(),
            bucket_name: "raven".to_owned(),
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
    let result: Result<Vec<Request>, String> = raven_config.create_crawler_requests();

    let expected_object_keys = vec![
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "1", "0", "100"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "2", "0", "100"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "1", "100", "200"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "1", "300", "200"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "2", "100", "200"),
        format!("test/{}/{}_{}_{}.html", &now_y_m_d, "2", "300", "200"),
    ];

    assert_eq!(result.is_ok(), true);
    let requests: Vec<Request> = result.unwrap();
    assert_eq!(requests.len(), 6);
    for request in requests.iter() {
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.header.len(), 0);
        assert_eq!(request.input_charset, "UTF-8");
        assert_eq!(request.output_charset, "UTF-8");
        assert_eq!(request.timeout, 5);
        assert_eq!(request.max_retry, 1);
        assert_eq!(request.body_params.len(), 0);
        assert_eq!(expected_url.contains(&request.url), true);
        let file_name = match &request.ouput_methods[0] {
            OutputMethod::LocalFile { .. } => panic!("must not be LocalFile."),
            OutputMethod::AmazonS3 { object_key, .. } => object_key,
        };
        assert_eq!(expected_object_keys.contains(&file_name), true);
    }
}

fn parse_key_value_map(map: &HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    if map.is_empty() {
        return vec![HashMap::new()];
    }

    let now: DateTime<Local> = Local::now();

    let mut single_map_lists: Vec<Vec<HashMap<String, String>>> = vec![];
    for (key, values) in map.into_iter() {
        let single_maps = values
            .iter()
            .map(|val| now.format(val).to_string())
            .flat_map(|val| try_expand_list(&val))
            .map(|val| {
                let mut single_map = HashMap::new();
                single_map.insert(key.to_owned(), val);
                single_map
            })
            .collect::<Vec<HashMap<String, String>>>();
        single_map_lists.push(single_maps);
    }

    single_map_lists
        .into_iter()
        .fold(vec![HashMap::new()], |result_list, list_of_map| {
            let mut new_result_list: Vec<HashMap<String, String>> = Vec::new();
            for result_item in &result_list {
                for map in &list_of_map {
                    let mut result_map: HashMap<String, String> = HashMap::new();
                    result_map.extend(result_item.to_owned());
                    result_map.extend(map.to_owned());
                    new_result_list.push(result_map);
                }
            }
            new_result_list
        })
}

#[test]
fn parse_key_value_map_and_template_parser_test() {
    let now_y_m_d = Local::now().format("%Y-%m-%d").to_string();

    let builder = TemplateBuilder::new("https://raven/{{a}}/{{b}}/{{c}}");

    let map = hashmap![
        "a".to_owned() => vec!["a1".to_owned(), "a2".to_owned()],
        "b".to_owned() => vec!["b[1..2]".to_owned()],
        "c".to_owned() => vec!["c1-%Y-%m-%d".to_owned()]
    ];

    let expected = vec![
        ["https://raven/a1/b1/c1-", &now_y_m_d].concat(),
        ["https://raven/a1/b2/c1-", &now_y_m_d].concat(),
        ["https://raven/a2/b1/c1-", &now_y_m_d].concat(),
        ["https://raven/a2/b2/c1-", &now_y_m_d].concat(),
    ];

    let parsed_map_list = parse_key_value_map(&map);
    assert_eq!(parsed_map_list.len(), expected.len());

    for parsed in parsed_map_list {
        let embedded = builder.build_string(&parsed).unwrap();
        assert!(expected.contains(&embedded));
    }
}
