// https://serde.rs
// https://serde.rs/attributes.html#container-attributes
extern crate log;
extern crate raven;
extern crate serde_yaml;

use raven::application::command_runner::config::config::RavenConfig;
use raven::application::command_runner::instances::Prd;
use raven::application::command_runner::runner::run_raven_application;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let config_file_path: String = env::args()
        .collect::<Vec<String>>()
        .get(1)
        .map(|path| path.to_owned())
        .expect("please pass config yaml file path.");

    let config_yaml_string = read_config_content(&config_file_path)
        .expect(&format!("cannot read file: {}", &config_file_path));

    let config = serde_yaml::from_str::<RavenConfig>(&config_yaml_string);

    let app = Prd::init(config.unwrap());

    run_raven_application(app);
}

fn read_config_content(path: &str) -> std::io::Result<String> {
    let config_file = File::open(path)?;
    let mut buf_reader = BufReader::new(config_file);
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;
    Ok(content)
}
