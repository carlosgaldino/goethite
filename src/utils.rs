use walkdir::DirEntry;
use std::path::{ Path, PathBuf };
use pulldown_cmark::{ html, Parser, self };
use std::io::prelude::*;
use std::fs::{ self, File };
use config::Config;
use regex::Regex;
use chrono::{ NaiveDate, UTC };
use rustc_serialize::{ Encodable, Encoder };
use std::ffi::OsStr;
use error::{ Result, GoethiteError };

#[derive(Clone, Debug)]
pub enum Markup {
    Markdown,
    HTML,
    Mustache,
}

impl Encodable for Markup {
    fn encode<S: Encoder>(&self, s: &mut S) -> ::std::result::Result<(), S::Error> {
        match *self {
            Markup::Markdown => try!(s.emit_str("markdown")),
            Markup::HTML     => try!(s.emit_str("html")),
            Markup::Mustache => try!(s.emit_str("mustache")),
        }
        Ok(())
    }
}

pub fn open_file<P: AsRef<Path>>(path: P) -> Result<File> {
    let path = path.as_ref();

    let file = match File::open(path) {
        Ok(f)    => f,
        Err(err) => match err.kind() {
            ::std::io::ErrorKind::NotFound => return Err(GoethiteError::NotFound(format!("{:?}", path))),
            _ => return Err(GoethiteError::from(err)),
        }
    };

    Ok(file)
}

pub struct Content {
    pub text:       String,
    pub attributes: Option<String>,
}

pub fn read_content(entry: &DirEntry) -> Result<Content> {
    let mut file   = try!(open_file(entry.path()));
    let mut buffer = String::new();
    try!(file.read_to_string(&mut buffer));

    let content: Vec<&str> = buffer.split("---").skip_while(|s| s.is_empty()).collect();

    if content.len() == 2 {
        Ok(Content { attributes: Some(content[0].to_string()), text: content[1].to_string() })
    } else {
        Ok(Content { attributes: None, text: content[0].to_string() })
    }
}

// TODO: remove this and use Path::relative_from when stable
fn normalize_path_str(s: &String) -> String {
    let s = s.to_owned();

    if s.ends_with("/") {
        s
    } else {
        s + "/"
    }
}

pub fn new_path<P: AsRef<Path>>(path: &P, prefix: Option<String>, config: &Config) -> PathBuf {
    let path              = path.as_ref();
    let file_name         = path.file_name().unwrap_or(OsStr::new(""));
    let normalized_source = normalize_path_str(&config.source);
    let rp                = path.to_str().unwrap().replace(&normalized_source, "");
    let base              = PathBuf::from(&config.destination);

    let relativized_path = if path.starts_with(normalized_source) {
        Path::new(&rp)
    } else {
        path
    };

    let new_path = match prefix {
        Some(p) => base.join(p).join(file_name),
        None    => base.join(relativized_path)
    };

    new_path
}

pub fn create_output_file(path: &PathBuf) -> Result<File> {
    try!(fs::create_dir_all(path.parent().unwrap()));

    let file = try!(File::create(path.with_extension("html")));

    Ok(file)
}

pub fn render_markdown(text: String) -> String {
    let mut rendered = String::new();
    let flags        = pulldown_cmark::OPTION_ENABLE_TABLES | pulldown_cmark::OPTION_ENABLE_FOOTNOTES;

    html::push_html(&mut rendered, Parser::new_ext(&text, flags));

    rendered
}

pub fn copy_file(entry: &DirEntry, config: &Config) -> Result<()> {
    let new_path = new_path(&entry.path(), None, &config);

    try!(fs::create_dir_all(new_path.parent().unwrap()));
    try!(fs::copy(entry.path(), new_path));

    Ok(())
}

pub fn slugify(str: &String) -> String {
    let mut re     = Regex::new(r"[^[:alnum:]]+").unwrap();
    let slug_title = re.replace_all(str, "-");
    re             = Regex::new(r"^-|-$").unwrap();

    re.replace_all(&slug_title, "").to_lowercase()
}

pub fn parse_date(str: Option<String>) -> Result<NaiveDate> {
    let date = match str {
        Some(date) => try!(NaiveDate::parse_from_str(&date, "%Y-%m-%d")),
        None       => UTC::today().naive_local(),
    };

    Ok(date)
}

pub fn extract_markup(path: &Path) -> Option<Markup> {
    match path.extension() {
        Some(ext) => match ext.to_str().unwrap() {
            "md" | "markdown" => Some(Markup::Markdown),
            "html"            => Some(Markup::HTML),
            "mustache"        => Some(Markup::Mustache),
            _                 => None,
        },
        None => None,
    }
}
