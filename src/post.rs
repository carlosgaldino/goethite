use toml;

#[derive(Debug)]
pub enum Attribute {
    Title(String),
    Author(String),
    Layout(String)
}

#[derive(Debug)]
pub struct Post {
    pub attributes: Vec<Attribute>,
    pub text: String
}

impl Post {
    pub fn new(buf: String) -> Post {
        let content: Vec<&str> = buf.split("---").skip_while(|s| s.is_empty()).collect();

        let post = if content.len() < 2 {
            Post { attributes: vec![], text: content[0].to_string() }
        } else {
            Post { attributes: parse_attributes(content[0]), text: content[1].to_string() }
        };

        post
    }
}

fn parse_attributes(str: &str) -> Vec<Attribute> {
    let mut parser = toml::Parser::new(str);
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
