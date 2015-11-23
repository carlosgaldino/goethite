use walkdir::{ DirEntry };
use std::path::{ Path, PathBuf };
use pulldown_cmark::{ html, Parser };
use std::io::prelude::*;
use std::fs::{ self, File };
use site::Site;

pub fn read_content(entry: &DirEntry) -> String {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    buffer
}

pub fn new_path(path: &Path, site: &Site) -> PathBuf {
    let path = path.to_str().unwrap().replace(&site.source, "");

    Path::new(&site.destination).join(path).with_extension("html")
}

// TODO: Not use `unwrap`
pub fn create_output_file(path: &PathBuf) -> File {
    fs::create_dir_all(path.parent().unwrap());

    File::create(path).unwrap()
}

pub fn render_markdown(text: &str) -> String {
    let mut rendered = String::new();
    html::push_html(&mut rendered, Parser::new(text));

    rendered
}
