use super::super::logger::log_level::LogLevel;
use serde_derive::*;
use std::collections::HashMap;
use chrono::{Local, DateTime};
use super::super::notify::Notify;
use super::request::RavenRequest;
use super::super::output::OutputMethod;

use super::raven_template_parser::*;
use super::super::crawl::*;

#[derive(Debug, PartialEq, Deserialize)]
pub struct RavenConfig {
    pub name: String,

    pub request: RavenRequest,

    #[serde(default)]
    pub notify: Vec<Notify>,

    pub output: Vec<OutputMethod>,

    #[serde(default = "default_max_threads")]
    pub max_threads: u16,

    pub log: LogConfig
}

fn default_max_threads() -> u16 {
    1
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LogConfig {
    pub file_path: String,
    pub level: LogLevel
}

macro_rules! copy_ref {
    ( $src:ident, $dest:ident ) => {
        for (key, val) in $src.iter() {
            std::collections::HashMap::insert(&mut $dest, key, val);
        }
    };
}

impl RavenConfig {
    pub fn create_crawler_requests(&self) -> Result<Vec<Request>, String> {
        let var_maps_list = self.request.vars.iter()
            .flat_map(|var_map| parse_key_value_map(var_map))
            .collect::<Vec<HashMap<String, String>>>();

        let param_map_list = self.request.params.iter()
            .flat_map(|params| parse_key_value_map(params))
            .collect::<Vec<HashMap<String, String>>>();
        
        let mut request_list: Vec<Request> = Vec::with_capacity(var_maps_list.len() * param_map_list.len());
                
        for (var_map, param_map) in product_list(&var_maps_list, &param_map_list) {
            request_list.push(self.create_crawler_request(var_map, param_map)?);
        }
        Ok(request_list)
    }


    pub fn create_crawler_request(&self, var_map: HashMap<String, String>, param_map: HashMap<String, String>) -> Result<Request, String> {
        let mut all_val_map: HashMap<&str, &str> = HashMap::new();
        copy_ref!(var_map, all_val_map);
        copy_ref!(param_map, all_val_map);

        let url = TemplateBuilder::new(&self.request.url).build_string(&all_val_map)?;
        let mut output_method: Vec<OutputMethod> = Vec::new();
        for method in &self.output {
            let embeded_output_method = match method {
                OutputMethod::LocalFile{ file_path } => 
                    OutputMethod::LocalFile {
                        file_path: TemplateBuilder::new(file_path).build_string(&all_val_map)?
                    },

                OutputMethod::AmazonS3{object_key, bucket_name, region} => 
                    OutputMethod::AmazonS3{
                        region: region.to_owned(),
                        bucket_name: bucket_name.to_owned(),
                        object_key: TemplateBuilder::new(object_key).build_string(&all_val_map)?
                    }
            };
            output_method.push(embeded_output_method);
        }

        if self.request.method == Method::Get {
            let request = Request {
                url: url,
                method: self.request.method.clone(),
                header: self.request.headers.clone(),
                ouput_methods: output_method,
                input_charset: self.request.input_charset.to_owned(),
                output_charset: self.request.output_charset.to_owned(),
                timeout: self.request.timeout_in_seconds,
                max_retry: self.request.max_retry,
                val_map: var_map,
                query_params: param_map,
                body_params: HashMap::new(),
            };
            Ok(request)
        } else {
            let request = Request {
                url: url,
                method: self.request.method.clone(),
                header: self.request.headers.clone(),
                ouput_methods: output_method,
                input_charset: self.request.input_charset.to_owned(),
                output_charset: self.request.output_charset.to_owned(),
                timeout: self.request.timeout_in_seconds,
                max_retry: self.request.max_retry,
                val_map: var_map,
                query_params: HashMap::new(),
                body_params: param_map
            };
            Ok(request)
        }
    }
}


fn parse_key_value_map(map: &HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    if map.is_empty() {
        return vec![HashMap::new()];
    }

    let now: DateTime<Local> = Local::now();

    let mut single_map_lists: Vec<Vec<HashMap<String, String>>> = vec![];
    for (key, values) in map.into_iter() {
        let single_maps = values.iter()
            .map(|val| now.format(val).to_string())
            .flat_map(|val| try_expand_list(&val))
            .map(|val| { 
                let mut single_map = HashMap::new();
                single_map.insert(key.to_owned(), val);
                single_map
            })
            .collect::<Vec<HashMap<String, String>>>();
        single_map_lists.push(single_maps);
    };

    single_map_lists.into_iter()
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

    let mut map = HashMap::new();
    map.insert("a".to_owned(), vec!["a1".to_owned(), "a2".to_owned()]);
    map.insert("b".to_owned(), vec!["b[1..2]".to_owned()]);
    map.insert("c".to_owned(), vec!["c1-%Y-%m-%d".to_owned()]);

    let expected = vec![
        ["https://raven/a1/b1/c1-", &now_y_m_d].concat(),
        ["https://raven/a1/b2/c1-", &now_y_m_d].concat(),
        ["https://raven/a2/b1/c1-", &now_y_m_d].concat(),
        ["https://raven/a2/b2/c1-", &now_y_m_d].concat()        
    ];

    let parsed_map_list = parse_key_value_map(&map);
    assert_eq!(parsed_map_list.len(), expected.len());
    
    for parsed in parsed_map_list {
        let embded = builder.build_string(&parsed).unwrap();
        assert!(expected.contains(&embded));
    };
}
