use std::io::prelude::*;
use std::fs::{ self, File };
use std::path::{ Path, PathBuf };
use pulldown_cmark::{ html, Parser };
use walkdir::{ DirEntry, WalkDir };
use post::Post;
use rustc_serialize::json::{ Json, ToJson };
use std::collections::HashMap;
use mustache;

struct PPost {
    post: Post,
    path: PathBuf
}

pub fn build(source: String, destination: String) {
    let walker = WalkDir::new(&source).into_iter();
    let mut posts = Vec::<PPost>::new();
    let mut templates: HashMap<String, mustache::Template> = HashMap::new();

    fs::remove_dir_all(&destination);

    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            match ext.to_str().unwrap() {
                "md" => posts.push(build_post(&entry, &source, &destination)),
                "mustache" => add_template(&entry, &mut templates),
                _ => {
                    let path     = entry.path().to_str().unwrap().replace(&source, "");
                    let new_path = Path::new(&destination).join(path);
                    fs::create_dir_all(new_path.parent().unwrap());

                    fs::copy(entry.path(), new_path);
                }
            }
        }
    }

    for post in posts {
        create_post(&post, &templates);
    }
}

fn add_template(entry: &DirEntry, templates: &mut HashMap<String, mustache::Template>) {
    let file_name = entry.file_name().to_str().unwrap().replace(".mustache", "").to_string();
    let template = mustache::compile_path(entry.path());

    templates.insert(file_name, template.unwrap());
}

fn build_post<'a>(entry: &DirEntry, source: &String, destination: &String) -> PPost {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let path     = entry.path().to_str().unwrap().replace(&source, "");
    let new_path = Path::new(&destination).join(path).with_extension("html");

    PPost { post: Post::new(buffer), path: new_path }
}

fn create_post(post: &PPost, templates: &HashMap<String, mustache::Template>) {
    let mut rendered = String::new();
    html::push_html(&mut rendered, Parser::new(&post.post.text));

    let data = mustache::MapBuilder::new()
        .insert_str("content", rendered)
        .insert_str("title", "Yo title")
        .insert_str("tagline", "Yo tagline")
        .build();

    fs::create_dir_all(post.path.parent().unwrap());

    let mut file = File::create(&post.path).unwrap();
    templates.get("page").unwrap().render_data(&mut file, &data);
}
