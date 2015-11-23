use toml;
use rustc_serialize::{ Encodable, Encoder, Decodable };
use std::path::{ PathBuf };
use site;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Attributes {
    title:  String,
    author: String,
    layout: String
}

#[derive(Debug, RustcEncodable)]
pub struct Post {
    pub attributes: Attributes,
    pub content:    String,
    pub path:       PathBuf,
}

impl Post {
    pub fn new(attributes: String, content: String, path: PathBuf, config: &site::Config) -> Post {
        Post { content: content, path: path, attributes: parse_attributes(attributes, &config) }
    }
}

fn parse_attributes(str: String, config: &site::Config) -> Attributes {
    let attrs: TomlAttributes = toml::decode_str(&str).unwrap();

    Attributes {
        layout: attrs.layout.unwrap_or(String::from("post")),
        author: attrs.author.unwrap_or(config.author.clone()),
        title:  attrs.title,
    }
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct TomlAttributes {
    title:  String,
    author: Option<String>,
    layout: Option<String>
}
