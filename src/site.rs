use std::io::prelude::*;
use std::fs::{ self, File };
use std::path::{ Path, PathBuf };
use pulldown_cmark::{ html, Parser };
use walkdir::{ DirEntry, WalkDir };
use post::Post;
use std::collections::HashMap;
use mustache;

struct Page {
    content: String,
    path: PathBuf,
}

pub fn build(source: String, destination: String) {
    let walker = WalkDir::new(&source).into_iter();
    let mut posts = Vec::<Post>::new();
    let mut templates: HashMap<String, mustache::Template> = HashMap::new();
    let mut pages: Vec<Page> = Vec::new();

    fs::remove_dir_all(&destination);

    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            match ext.to_str().unwrap() {
                "md" => posts.push(build_post(&entry, &source, &destination)),
                "mustache" => add_template(&entry, &mut templates),
                "html" => pages.push(build_page(&entry, &source, &destination)),
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

    for page in pages {
        create_page(&page, &templates);
    }
}

fn build_page(entry: &DirEntry, source: &String, destination: &String) -> Page {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let path     = entry.path().to_str().unwrap().replace(&source, "");
    let new_path = Path::new(&destination).join(path).with_extension("html");

    Page { content: buffer, path: new_path }
}

fn add_template(entry: &DirEntry, templates: &mut HashMap<String, mustache::Template>) {
    let file_name = entry.file_name().to_str().unwrap().replace(".mustache", "").to_string();
    let template = mustache::compile_path(entry.path());

    templates.insert(file_name, template.unwrap());
}

fn build_post(entry: &DirEntry, source: &String, destination: &String) -> Post {
    let mut file   = File::open(entry.path()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let content: Vec<&str> = buffer.split("---").skip_while(|s| s.is_empty()).collect();

    let rendered_content = render_markdown(content[1]);

    let path = entry.path().to_str().unwrap().replace(&source, "");
    let path = Path::new(&destination).join(path).with_extension("html");

    let post = Post::new(content[0].to_string(), rendered_content, path);

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
