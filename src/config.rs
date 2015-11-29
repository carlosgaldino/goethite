use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use toml;

#[derive(RustcEncodable)]
pub struct Config {
    pub author:      String,
    pub name:        String,
    pub source:      String,
    pub destination: String,
    tagline:         Option<String>,
    description:     Option<String>,
}

#[derive(RustcDecodable)]
struct TomlConfig {
    pub author:      String,
    pub name:        String,
    tagline:         Option<String>,
    description:     Option<String>,
}

impl Config {
    pub fn new(source: String, destination: String) -> Config {
        let path = Path::new(&source).join("goethite.toml");

        // TODO: handle the case when the config does not exist
        let mut file = File::open(path).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer);

        let config: Option<TomlConfig> = toml::decode_str(&buffer);

        let config = match config {
            Some(c) => c,
            None => panic!("invalid config"),
        };

        Config { author: config.author,
                 name: config.name,
                 tagline: config.tagline,
                 description: config.description,
                 source: source,
                 destination: destination }
    }
}
