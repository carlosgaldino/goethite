use toml;
use rustc_serialize::{ Encodable, Encoder };
use std::path::{ Path, PathBuf };
use site::Config;
use chrono::NaiveDate;
use utils;

#[derive(RustcEncodable, Debug, Clone)]
pub struct Attributes {
    title:      String,
    author:     String,
    pub layout: String,
    permalink:  String,
    date:       NaiveDate,
}

#[derive(Clone, Debug)]
pub enum Markup {
    Markdown,
    HTML
}

impl Encodable for Markup {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            Markup::Markdown => try!(s.emit_str("markdown")),
            Markup::HTML     => try!(s.emit_str("html")),
        }
        Ok(())
    }
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
        let path       = utils::new_path(&attributes.permalink, config);

        Page {
            content:    content,
            path:       path.clone(),
            attributes: attributes,
            markup:     extract_markup(path),
        }
    }

    pub fn is_post(&self) -> bool {
        self.attributes.layout == String::from("post")
    }
}

// TODO: move to utils and return Option<Markup>
fn extract_markup(path: PathBuf) -> Markup {
    let ext = path.extension().unwrap().to_str().unwrap();

    match ext {
        "md" | "markdown" => Markup::Markdown,
        "html"            => Markup::HTML,
        _                 => panic!("Unknown markup"),
    }
}

fn build_attrs(attrs: TomlAttributes, path: &Path, config: &Config) -> Attributes {
    let layout = attrs.layout.unwrap_or(String::from("page"));
    let title  = attrs.title.unwrap_or(path.file_stem().unwrap().to_str().unwrap().to_string());
    let date   = utils::parse_date(attrs.date);

    let permalink = if layout == "post" {
        format!("{}/{}.html", date.format("%Y/%m/%d"), utils::slugify(&title))
    } else {
        format!("{}.html", utils::slugify(&title))
    };

    Attributes {
        author:    attrs.author.unwrap_or(config.author.clone()),
        layout:    layout,
        title:     title,
        date:      date,
        permalink: permalink,
    }
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct TomlAttributes {
    title:  Option<String>,
    author: Option<String>,
    layout: Option<String>,
    date:   Option<String>,
}
