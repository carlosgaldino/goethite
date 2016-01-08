use std::path::Path;
use toml;
use error::{GoethiteError, Result};
use utils;

#[derive(RustcEncodable)]
pub struct Config {
    pub author: String,
    pub name: String,
    pub source: String,
    pub destination: String,
    tagline: Option<String>,
    description: Option<String>,
}

#[derive(RustcDecodable)]
struct TomlConfig {
    author: String,
    name: String,
    tagline: Option<String>,
    description: Option<String>,
}

impl Config {
    pub fn new(source: String, destination: String) -> Result<Config> {
        let path = Path::new(&source).join("goethite.toml");
        let buffer = try!(utils::read_file(path));

        let config: Option<TomlConfig> = toml::decode_str(&buffer);

        let config = match config {
            Some(c) => c,
            None => return Err(GoethiteError::InvalidConfig),
        };

        let c = Config { author: config.author,
                         name: config.name,
                         tagline: config.tagline,
                         description: config.description,
                         source: source,
                         destination: destination, };

        Ok(c)
    }
}
