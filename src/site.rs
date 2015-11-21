use std::io::prelude::*;
use std::fs::{ self, File };
use std::path::Path;
use pulldown_cmark::{ html, Parser };
use walkdir::{ DirEntry, WalkDir };
use post::Post;

pub fn build(source: String, destination: String) {
    let walker = WalkDir::new(&source).into_iter();

    fs::remove_dir_all(&destination);

    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            match ext.to_str().unwrap() {
                "md" => create_post(&entry, &source, &destination),
                _ => {
                    let path     = entry.path().to_str().unwrap().replace(&source, "");
                    let new_path = Path::new(&destination).join(path);

                    fs::copy(entry.path(), new_path);
                }
            }
        }
    }
}

fn create_post(entry: &DirEntry, source: &String, destination: &String) {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let post = Post::create(&buffer);

    let mut rendered = String::new();
    html::push_html(&mut rendered, Parser::new(post.text));

    let path     = entry.path().to_str().unwrap().replace(&source, "");
    let new_path = Path::new(&destination).join(path).with_extension("html");

    fs::create_dir_all(&new_path.parent().unwrap());

    file = File::create(new_path).unwrap();
    file.write_all(rendered.as_bytes());
}
