// https://serde.rs
// https://serde.rs/attributes.html#container-attributes
extern crate log;
extern crate raven;
extern crate serde_yaml;

use raven::application::*;
use raven::input::RavenConfig;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let config_yaml_string = read_config_content().unwrap();

    println!("{}", config_yaml_string);

    let config = serde_yaml::from_str::<RavenConfig>(&config_yaml_string);

    println!("{:?}", config);

    run_raven_application::<Prd>(config.unwrap());
}

fn read_config_content() -> std::io::Result<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        panic!("please pass config file.");
    }

    let config_file = File::open(&args[1])?;
    let mut buf_reader = BufReader::new(config_file);
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;
    Ok(content)
}
