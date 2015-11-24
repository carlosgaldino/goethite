use walkdir::{ DirEntry };
use std::path::{ Path, PathBuf };
use pulldown_cmark::{ html, Parser };
use std::io::prelude::*;
use std::fs::{ self, File };
use site::Config;
use regex::Regex;
use chrono::{ NaiveDate, UTC };

pub fn read_content(entry: &DirEntry) -> (String, String) {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let content: Vec<&str> = buffer.split("---").skip_while(|s| s.is_empty()).collect();

    if content.len() < 2 {
        panic!("Front Matter not found for {:?}", &entry.path());
    }

    (content[0].to_string(), content[1].to_string())
}

pub fn new_path(path: &str, config: &Config) -> PathBuf {
    let path = path.replace(&config.source, "");

    Path::new(&config.destination).join(path)
}

// TODO: Not use `unwrap`
pub fn create_output_file(path: &PathBuf) -> File {
    fs::create_dir_all(path.parent().unwrap());

    File::create(path.with_extension("html")).unwrap()
}

pub fn render_markdown(text: String) -> String {
    let mut rendered = String::new();
    html::push_html(&mut rendered, Parser::new(&text));

    rendered
}

pub fn copy_file(entry: &DirEntry, config: &Config) {
    let new_path = new_path(entry.path().to_str().unwrap(), &config);

    fs::create_dir_all(new_path.parent().unwrap());
    fs::copy(entry.path(), new_path);
}

pub fn slugify(str: &String) -> String {
    let mut re     = Regex::new(r"[^[:alnum:]]+").unwrap();
    let slug_title = re.replace_all(str, "-");
    re             = Regex::new(r"^-|-$").unwrap();

    re.replace_all(&slug_title, "").to_lowercase()
}

pub fn parse_date(str: Option<String>) -> NaiveDate {
    match str {
        Some(date) => match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => panic!("Invalid date format"),
        },
        None => UTC::today().naive_local(),
    }
}
