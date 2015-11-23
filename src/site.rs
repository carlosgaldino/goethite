use std::fs;
use std::path::{ Path };
use walkdir::{ DirEntry, WalkDir };
use post::Post;
use page::Page;
use std::collections::HashMap;
use mustache;
use utils;

type Templates = HashMap<String, mustache::Template>;

#[derive(RustcEncodable)]
pub struct Config {
    pub author: String,
    pub name:   String,
}

#[derive(RustcEncodable)]
struct Context<'a> {
    site: &'a Site,
    post: &'a Post
}

#[derive(RustcEncodable)]
pub struct Site {
    pub source: String,
    pub destination: String,
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
    let site = Site::new(source, destination, Config { author: String::from("Carlos Galdino"), name: String::from("cg") });

    let walker                   = WalkDir::new(&site.source).into_iter();
    let mut posts: Vec<Post>     = Vec::new();
    let mut templates: Templates = HashMap::new();
    let mut pages: Vec<Page>     = Vec::new();

    fs::remove_dir_all(&site.destination);

    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            match ext.to_str().unwrap() {
                "md"       => posts.push(build_post(&entry, &site)),
                "mustache" => add_template(&entry, &mut templates),
                "html"     => pages.push(build_page(&entry, &site)),
                _ => {
                    let path     = entry.path().to_str().unwrap().replace(&site.source, "");
                    let new_path = Path::new(&site.destination).join(path);
                    fs::create_dir_all(new_path.parent().unwrap());

                    fs::copy(entry.path(), new_path);
                }
            }
        }
    }

    let site = Site { posts: posts, pages: pages, .. site };

    for post in &site.posts {
        create_post(&post, &site, &templates);
    }

    for page in &site.pages {
        create_page(&page, &site, &templates);
    }
}

fn build_page(entry: &DirEntry, site: &Site) -> Page {
    let content = utils::read_content(&entry);

    Page { content: content, path: utils::new_path(&entry.path(), &site) }
}

fn build_post(entry: &DirEntry, site: &Site) -> Post {
    let content = utils::read_content(&entry);

    let content: Vec<&str> = content.split("---").skip_while(|s| s.is_empty()).collect();
    let rendered_content   = utils::render_markdown(content[1]);
    let path               = utils::new_path(&entry.path(), &site);
    let post               = Post::new(content[0].to_string(), rendered_content, path, &site.config);

    post
}

fn add_template(entry: &DirEntry, templates: &mut Templates) {
    let file_name = entry.file_name().to_str().unwrap().replace(".mustache", "").to_string();
    let template = mustache::compile_path(entry.path());

    templates.insert(file_name, template.unwrap());
}

fn create_post(post: &Post, site: &Site, templates: &Templates) {
    let mut file = utils::create_output_file(&post.path);
    let context  = Context { site: site, post: post };

    templates.get("post").unwrap().render(&mut file, &context);
}

fn create_page(page: &Page, site: &Site, templates: &Templates) {
    let page_template         = mustache::compile_str(&page.content);
    let mut rendered: Vec<u8> = Vec::new();
    page_template.render(&mut rendered, &site);

    let data = mustache::MapBuilder::new()
        .insert_str("content", String::from_utf8(rendered).unwrap())
        .insert_str("title", "Yo title")
        .insert_str("tagline", "Yo tagline")
        .build();

    let mut file = utils::create_output_file(&page.path);
    templates.get("page").unwrap().render_data(&mut file, &data);
}
