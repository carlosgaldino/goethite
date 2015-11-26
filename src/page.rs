use toml;
use rustc_serialize::{ Encodable, Encoder };
use std::path::{ Path, PathBuf };
use site::Config;
use chrono::NaiveDate;
use utils::{ self, Markup };

#[derive(RustcEncodable, Debug, Clone)]
pub struct Attributes {
    title:      String,
    author:     String,
    pub layout: String,
    permalink:  String,
    prefix:     Option<String>,
    pub date:   NaiveDate,
}

#[derive(Debug, RustcEncodable, Clone)]
pub struct Page {
    pub attributes: Attributes,
    pub content:    String,
    pub path:       PathBuf,
    pub markup:     Markup,
}

impl Page {
    pub fn new(attributes: String, content: String, path: &Path, config: &Config) -> Page {
        let attrs: Option<TomlAttributes> = toml::decode_str(&attributes);

        let attrs = match attrs {
            Some(attrs) => attrs,
            None        => panic!("Invalid front matter for {:?}", &path),
        };

        let attributes = build_attrs(attrs, path, config);
        let new_path   = utils::new_path(&attributes.permalink, attributes.prefix.clone(), config);
        let markup     = utils::extract_markup(path).expect("Unknown markup");

        Page {
            content:    content,
            path:       new_path,
            attributes: attributes,
            markup:     markup,
        }
    }

    pub fn is_post(&self) -> bool {
        self.attributes.layout == String::from("post")
    }
}

fn build_attrs(attrs: TomlAttributes, path: &Path, config: &Config) -> Attributes {
    let file_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
    let layout    = attrs.layout.unwrap_or(String::from("page"));
    let title     = attrs.title.unwrap_or(file_stem.clone());
    let date      = utils::parse_date(attrs.date);

    let (prefix, permalink) = if layout == "post" {
        let prefix    = date.format("%Y/%m/%d");
        let permalink = format!("{}/{}.html", prefix, utils::slugify(&file_stem));

        (Some(prefix.to_string()), permalink)
    } else {
        (None, format!("{}.html", file_stem))
    };

    Attributes {
        author:    attrs.author.unwrap_or(config.author.clone()),
        layout:    layout,
        title:     title,
        date:      date,
        permalink: permalink,
        prefix:    prefix,
    }
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct TomlAttributes {
    title:  Option<String>,
    author: Option<String>,
    layout: Option<String>,
    date:   Option<String>,
}
