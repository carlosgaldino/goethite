use std::io::prelude::*;
use std::fs::{ self, File };
use std::path::{ Path };
use pulldown_cmark::{ html, Parser };
use walkdir::{ DirEntry, WalkDir };
use post::Post;
use page::Page;
use std::collections::HashMap;
use mustache;

pub struct Config {
    pub author: String,
}

struct Site {
    source: String,
    destination: String,
    config: Config,
    posts: Vec<Post>,
    pages: Vec<Page>,
}

impl Site {
    fn new(source: String, destination: String, config: Config) -> Site {
        Site { source: source, destination: destination, config: config, posts: Vec::new(), pages: Vec::new() }
    }
}

pub fn build(source: String, destination: String) {
    let site = Site::new(source, destination, Config { author: String::from("Carlos Galdino") });

    let walker = WalkDir::new(&site.source).into_iter();
    let mut posts = Vec::<Post>::new();
    let mut templates: HashMap<String, mustache::Template> = HashMap::new();
    let mut pages: Vec<Page> = Vec::new();

    fs::remove_dir_all(&site.destination);

    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            match ext.to_str().unwrap() {
                "md" => posts.push(build_post(&entry, &site)),
                "mustache" => add_template(&entry, &mut templates),
                "html" => pages.push(build_page(&entry, &site)),
                _ => {
                    let path     = entry.path().to_str().unwrap().replace(&site.source, "");
                    let new_path = Path::new(&site.destination).join(path);
                    fs::create_dir_all(new_path.parent().unwrap());

                    fs::copy(entry.path(), new_path);
                }
            }
        }
    }

    for post in posts {
        create_post(&post, &templates);
    }

    for page in pages {
        create_page(&page, &templates);
    }
}

fn build_page(entry: &DirEntry, site: &Site) -> Page {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let path     = entry.path().to_str().unwrap().replace(&site.source, "");
    let new_path = Path::new(&site.destination).join(path).with_extension("html");

    Page { content: buffer, path: new_path }
}

fn add_template(entry: &DirEntry, templates: &mut HashMap<String, mustache::Template>) {
    let file_name = entry.file_name().to_str().unwrap().replace(".mustache", "").to_string();
    let template = mustache::compile_path(entry.path());

    templates.insert(file_name, template.unwrap());
}

fn build_post(entry: &DirEntry, site: &Site) -> Post {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let content: Vec<&str> = buffer.split("---").skip_while(|s| s.is_empty()).collect();

    let rendered_content = render_markdown(content[1]);

    let path = entry.path().to_str().unwrap().replace(&site.source, "");
    let path = Path::new(&site.destination).join(path).with_extension("html");

    let post = Post::new(content[0].to_string(), rendered_content, path, &site.config);

    post
}

fn render_markdown(text: &str) -> String {
    let mut rendered = String::new();
    html::push_html(&mut rendered, Parser::new(text));

    rendered
}

fn create_post(post: &Post, templates: &HashMap<String, mustache::Template>) {
    fs::create_dir_all(post.path.parent().unwrap());

    let mut file = File::create(&post.path).unwrap();
    templates.get("page").unwrap().render(&mut file, &post);
}

fn create_page(page: &Page, templates: &HashMap<String, mustache::Template>) {
    let page_data = mustache::MapBuilder::new()
        .insert_str("title", "Page title")
        .insert_str("tagline", "Page tagline")
        .build();

    let page_template = mustache::compile_str(&page.content);
    let mut rendered: Vec<u8> = Vec::new();
    page_template.render_data(&mut rendered, &page_data);

    let data = mustache::MapBuilder::new()
        .insert_str("content", String::from_utf8(rendered).unwrap())
        .insert_str("title", "Yo title")
        .insert_str("tagline", "Yo tagline")
        .build();

    fs::create_dir_all(page.path.parent().unwrap());

    let mut file = File::create(&page.path).unwrap();
    templates.get("page").unwrap().render_data(&mut file, &data);
}
