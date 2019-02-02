// https://serde.rs
// https://serde.rs/attributes.html#container-attributes
extern crate serde_yaml;
extern crate log;
extern crate raven;

use raven::input::RavenConfig;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::env;

fn main() {
    let config_yaml_string = read_config_content()
        .unwrap_or_else(|error| 
            panic!(error.to_string())
        );

    println!("{}", config_yaml_string);

    let config = serde_yaml::from_str::<RavenConfig>(&config_yaml_string);
    
    println!("{:?}", config);
}

fn read_config_content() -> std::io::Result<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 0 {
        panic!("please pass config file.");
    }

    let config_file = File::open(&args[1])?;
    let mut buf_reader = BufReader::new(config_file);
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;
    Ok(content)
}

