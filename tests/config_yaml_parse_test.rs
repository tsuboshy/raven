extern crate raven;
extern crate serde_yaml;

use raven::application::command_runner::config::config::RavenConfig;
use raven::application::command_runner::config::log::LogConfig;
use raven::application::core_types::crawler::request::Method::{Get, Post};
use raven::application::core_types::logger::LogLevel::{Debug, Warn};
use raven::application::core_types::notify_method::{Notify, NotifyMethod};
use raven::application::core_types::persist::PersistMethod;
use raven::charset::Charset;

static FULL_PARAMETER_YAML: &'static str = r#"
name: "テスト"
request:
  url: "https://www.craw.app/{{id}}/{{number}}"
  vars:
    - id: 
        - "[1..10]"
      number: 
        - "1"
  method: Post
  headers:
    User-Agent: "raven"
    Content-Type: "application/json"
  params:
    - offset:
        - "100"
        - "300"
        - "500"
      limit:
        - "200"

    - offset:
        - "0"
      limit: 
        - "100"
  timeout_in_seconds: 30
  max_retry: 5
  encoding:
    input: "UTF-8"
    output: "UTF-8"

max_threads : 10


notify:
  - slack:
      url: "https://slack.service/xxxx"
      channel: "raven-devops"
      mention: "here"

output:
  - local_file:
      file_path: "/var/application/%Y/%m/%d/{{id}}.html"

  - amazon_s3:
      region: "ap-nothereast-1"
      bucket_name: "test_bucket"
      object_key: "test_key"

log:
  file_path: "/var/tmp/application.log"
  level: "warn"
"#;

#[test]
fn it_should_success_to_parse_when_full_parameter_exists() {
    let parsed = serde_yaml::from_str::<RavenConfig>(&FULL_PARAMETER_YAML).unwrap();
    assert_eq!(parsed.name, "テスト");
    assert_eq!(parsed.request.url, "https://www.craw.app/{{id}}/{{number}}");

    let vars = &parsed.request.vars;
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].get("id").unwrap()[0], "[1..10]");
    assert_eq!(vars[0].get("number").unwrap()[0], "1");
    assert_eq!(parsed.request.method, Post);

    let headers = &parsed.request.headers;
    assert_eq!(headers.get("User-Agent").unwrap(), "raven");
    assert_eq!(headers.get("Content-Type").unwrap(), "application/json");

    let params = &parsed.request.params;
    assert_eq!(params.len(), 2);
    let param_offset_0 = params[0].get("offset").unwrap();
    let param_limit_0 = params[0].get("limit").unwrap();
    assert_eq!(param_offset_0[0], "100");
    assert_eq!(param_offset_0[1], "300");
    assert_eq!(param_offset_0[2], "500");
    assert_eq!(param_limit_0[0], "200");
    let param_offset_1 = params[1].get("offset").unwrap();
    let param_limit_1 = params[1].get("limit").unwrap();
    assert_eq!(param_offset_1[0], "0");
    assert_eq!(param_limit_1[0], "100");

    assert_eq!(parsed.request.timeout_in_seconds, 30);
    assert_eq!(parsed.request.max_retry, 5);
    let encoding = parsed.request.encoding.unwrap();
    assert_eq!(encoding.input, Some(Charset::Utf8));
    assert_eq!(encoding.output, Charset::Utf8);

    assert_eq!(parsed.max_threads, 10);

    let notify = &parsed.notify;
    assert_eq!(notify.len(), 1);
    let expected_notify = NotifyMethod::Slack {
        url: "https://slack.service/xxxx".to_owned(),
        channel: "raven-devops".to_owned(),
        mention: Some("here".to_owned()),
    };
    assert_eq!(notify[0], expected_notify);

    let output = &parsed.output;
    assert_eq!(output.len(), 2);
    let expected_local = PersistMethod::LocalFile {
        file_path: "/var/application/%Y/%m/%d/{{id}}.html".to_owned(),
    };
    let expected_s3 = PersistMethod::AmazonS3 {
        region: "ap-nothereast-1".to_owned(),
        bucket_name: "test_bucket".to_owned(),
        object_key: "test_key".to_owned(),
    };
    assert_eq!(output[0], expected_local);
    assert_eq!(output[1], expected_s3);

    let expected_log_config = LogConfig {
        file_path: "/var/tmp/application.log".to_owned(),
        level: Warn,
    };
    assert_eq!(parsed.log, expected_log_config);
}

static MIN_CONFIG_YAML: &'static str = r#"
name: "テスト"
request:
  url: "https://www.craw.app/"
  method: Get

output:
  - local_file:
      file_path: "/var/application/%Y/%m/%d/{{id}}.html"

log:
  file_path: "/var/tmp/application.log"
  level: "DEBUG"
"#;

#[test]
fn it_should_success_to_parse_when_only_required_param_exists() {
    let parsed = serde_yaml::from_str::<RavenConfig>(&MIN_CONFIG_YAML).unwrap();
    assert_eq!(parsed.name, "テスト");
    assert_eq!(parsed.request.url, "https://www.craw.app/");

    let vars = &parsed.request.vars;
    assert_eq!(vars.len(), 0);
    assert_eq!(parsed.request.method, Get);

    let headers = &parsed.request.headers;
    assert_eq!(headers.len(), 0);

    let params = &parsed.request.params;
    assert_eq!(params.len(), 0);

    assert_eq!(parsed.request.timeout_in_seconds, 1);
    assert_eq!(parsed.request.max_retry, 0);
    assert_eq!(parsed.request.encoding, None);
    assert_eq!(parsed.max_threads, 1);

    let notify = &parsed.notify;
    assert_eq!(notify.len(), 0);

    let output = &parsed.output;
    assert_eq!(output.len(), 1);
    let expected_local = PersistMethod::LocalFile {
        file_path: "/var/application/%Y/%m/%d/{{id}}.html".to_owned(),
    };
    assert_eq!(output[0], expected_local);

    let expected_log_config = LogConfig {
        file_path: "/var/tmp/application.log".to_owned(),
        level: Debug,
    };
    assert_eq!(parsed.log, expected_log_config);
}
