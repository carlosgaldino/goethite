use toml;
use rustc_serialize::{ Encodable, Encoder };
use std::path::PathBuf;
use site;

#[derive(RustcEncodable, Debug, Clone)]
pub struct Attributes {
    title:      String,
    author:     String,
    pub layout: String,
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
    pub fn new(attributes: String, content: String, path: PathBuf, config: &site::Config) -> Page {
        let attrs: Option<TomlAttributes> = toml::decode_str(&attributes);

        let attrs = match attrs {
            Some(attrs) => attrs,
            None        => panic!("Front Matter not found for {:?}", &path),
        };

        Page {
            content:    content,
            path:       path.clone(),
            attributes: build_attrs(attrs, &config),
            markup:     extract_markup(path),
        }
    }

    pub fn is_post(&self) -> bool {
        self.attributes.layout == String::from("post")
    }
}

fn extract_markup(path: PathBuf) -> Markup {
    let ext = path.extension().unwrap().to_str().unwrap();

    match ext {
        "md" | "markdown" => Markup::Markdown,
        "html"            => Markup::HTML,
        _                 => panic!("Unknown markup"),
    }
}

fn build_attrs(attrs: TomlAttributes, config: &site::Config) -> Attributes {
    Attributes {
        author: attrs.author.unwrap_or(config.author.clone()),
        layout: attrs.layout.unwrap_or(String::from("page")),
        title:  attrs.title,
    }
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct TomlAttributes {
    title:  String,
    author: Option<String>,
    layout: Option<String>
}
