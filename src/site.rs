use std::io::prelude::*;
use std::fs::{ self, File };
use std::path::{ Path, PathBuf };
use pulldown_cmark::{ html, Parser };
use walkdir::{ DirEntry, WalkDir };
use post::Post;
use handlebars::{ Handlebars };
use rustc_serialize::json::{ Json, ToJson };
use std::collections::HashMap;

struct PPost {
    post: Post,
    path: PathBuf
}

pub fn build(source: String, destination: String) {
    let walker = WalkDir::new(&source).into_iter();
    let mut posts = Vec::<PPost>::new();
    let mut handlebars = Handlebars::new();

    fs::remove_dir_all(&destination);

    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            match ext.to_str().unwrap() {
                "md" => posts.push(build_post(&entry, &source, &destination)),
                "hbs" => register_template(&entry, &mut handlebars),
                _ => {
                    let path     = entry.path().to_str().unwrap().replace(&source, "");
                    let new_path = Path::new(&destination).join(path);

                    fs::copy(entry.path(), new_path);
                }
            }
        }
    }

    for post in posts {
        create_post(&post, &mut handlebars);
    }
}

fn build_post<'a>(entry: &DirEntry, source: &String, destination: &String) -> PPost {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let path     = entry.path().to_str().unwrap().replace(&source, "");
    let new_path = Path::new(&destination).join(path).with_extension("html");

    PPost { post: Post::new(buffer), path: new_path }
}

fn register_template(entry: &DirEntry, handlebars: &mut Handlebars) {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let template_name = Path::new(entry.file_name().to_str().unwrap()).file_stem().unwrap();

    handlebars.register_template_string(template_name.to_str().unwrap(), buffer);
}

fn create_post(post: &PPost, handlebars: &mut Handlebars) {
    let mut rendered = String::new();
    html::push_html(&mut rendered, Parser::new(&post.post.text));

    let mut data: HashMap<String, Json> = HashMap::new();
    data.insert("title".to_string(), "My Title".to_json());
    data.insert("tagline".to_string(), "My Tagline".to_json());

    handlebars.register_template_string("content", rendered);

    rendered = handlebars.render("page", &data).unwrap();

    fs::create_dir_all(post.path.parent().unwrap());

    let mut file = File::create(&post.path).unwrap();
    file.write_all(rendered.as_bytes());

    handlebars.unregister_template("content");
}
