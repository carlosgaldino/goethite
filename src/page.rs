use toml;
use rustc_serialize::{ Encodable, Encoder };
use std::path::{ Path, PathBuf };
use site::Config;
use chrono::NaiveDate;
use utils::{ self, Markup };

#[derive(Debug, Clone)]
pub struct Attributes {
    title:      String,
    author:     String,
    pub layout: String,
    permalink:  String,
    prefix:     Option<String>,
    pub date:   NaiveDate,
}

impl Encodable for Attributes {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_map(1, |e| {
            let mut i = 0;

            try!(e.emit_map_elt_key(i, |e| "title".encode(e)));
            try!(e.emit_map_elt_val(i, |e| self.title.encode(e)));
            i += 1;

            try!(e.emit_map_elt_key(i, |e| "author".encode(e)));
            try!(e.emit_map_elt_val(i, |e| self.author.encode(e)));
            i += 1;

            try!(e.emit_map_elt_key(i, |e| "layout".encode(e)));
            try!(e.emit_map_elt_val(i, |e| self.layout.encode(e)));
            i += 1;

            try!(e.emit_map_elt_key(i, |e| "permalink".encode(e)));
            try!(e.emit_map_elt_val(i, |e| self.permalink.encode(e)));
            i += 1;

            try!(e.emit_map_elt_key(i, |e| "prefix".encode(e)));
            try!(e.emit_map_elt_val(i, |e| self.prefix.encode(e)));
            i += 1;

            try!(e.emit_map_elt_key(i, |e| "date".encode(e)));
            try!(e.emit_map_elt_val(i, |e| self.date.format("%d %b %Y").to_string().encode(e)));

            Ok(())
        })
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
