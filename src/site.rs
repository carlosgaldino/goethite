use std::io::prelude::*;
use std::fs::{ self, File };
use std::path::Path;
use pulldown_cmark::{ html, Parser };
use walkdir::{ DirEntry, WalkDir };
use post::Post;

pub fn build(source: String, destination: String) {
    let walker = WalkDir::new(source).into_iter();

    for entry in walker.filter_map(|e| e.ok()).filter(|e| is_markdown(e)) {
        let mut file   = File::open(entry.path()).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer);

        let post = Post::create(&buffer);

        let mut rendered = String::new();
        html::push_html(&mut rendered, Parser::new(post.text));

        fs::create_dir_all(&destination);
        let out_path = Path::new(&destination).join(entry.file_name()).with_extension("html");
        file         = File::create(out_path).unwrap();
        file.write_all(rendered.as_bytes());
    }
}

fn is_markdown(entry: &DirEntry) -> bool {
    entry.path().extension().map(|ext| ["md", "markdown"].contains(&ext.to_str().unwrap())).unwrap_or(false)
}
