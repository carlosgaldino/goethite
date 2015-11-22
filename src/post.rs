use toml;
use rustc_serialize::{ Encodable, Encoder };
use std::path::{ PathBuf };

#[derive(Debug)]
pub enum Attribute {
    Title(String),
    Author(String),
    Layout(String)
}

impl Encodable for Attribute {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            Attribute::Title(ref title) => {
                try!(s.emit_map_elt_key(0, |e| "title".encode(e)));
                try!(s.emit_map_elt_val(0, |e| title.encode(e)));
            },
            Attribute::Author(ref author) => {
                try!(s.emit_map_elt_key(0, |e| "author".encode(e)));
                try!(s.emit_map_elt_val(0, |e| author.encode(e)));
            },
            Attribute::Layout(ref layout) => {
                try!(s.emit_map_elt_key(0, |e| "layout".encode(e)));
                try!(s.emit_map_elt_val(0, |e| layout.encode(e)));
            },
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Post {
    pub attributes: Vec<Attribute>,
    pub content: String,
    pub path: PathBuf,
}

impl Encodable for Post {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_map(1, |e| {
            let mut i = 0;
            try!(e.emit_map_elt_key(i, |e| "content".encode(e)));
            try!(e.emit_map_elt_val(i, |e| self.content.encode(e)));
            i += 1;

            try!(e.emit_map_elt_key(i, |e| "path".encode(e)));
            try!(e.emit_map_elt_val(i, |e| self.path.encode(e)));
            i += 1;

            try!(e.emit_map_elt_key(i, |e| "attributes".encode(e)));
            try!(e.emit_map_elt_val(i, |e| {
                e.emit_map(1, |e| {
                    for attr in &self.attributes {
                        attr.encode(e);
                    }
                    Ok(())
                })
            }));

            Ok(())
        })
    }
}

impl Post {
    pub fn new(attributes: String, content: String, path: PathBuf) -> Post {
        Post { content: content, path: path, attributes: parse_attributes(attributes) }
    }
}

fn parse_attributes(str: String) -> Vec<Attribute> {
    let mut parser = toml::Parser::new(&str);
    let mut attrs  = Vec::<Attribute>::new();

    match parser.parse() {
        Some(values) => {
            for (k, v) in values {
                // TODO: make it work with `match`
                if k == "title" {
                    attrs.push(Attribute::Title(String::from(v.as_str().unwrap())));
                } else if k == "layout" {
                    attrs.push(Attribute::Layout(String::from(v.as_str().unwrap())));
                } else if k == "author" {
                    attrs.push(Attribute::Author(String::from(v.as_str().unwrap())));
                }
            };
        }
        None => println!("errors: {:?}", parser.errors)
    }

    attrs
}
