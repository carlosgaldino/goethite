use std::fs;
use std::path::{ Path };
use walkdir::{ DirEntry, WalkDir };
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
    page: &'a Page
}

#[derive(RustcEncodable)]
pub struct Site {
    pub source: String,
    pub destination: String,
    config: Config,
    pages: Vec<Page>,
    posts: Vec<Page>,
}

impl Site {
    fn new(source: String, destination: String, config: Config) -> Site {
        Site { source: source, destination: destination, config: config, pages: Vec::new(), posts: Vec::new() }
    }
}

pub fn build(source: String, destination: String) {
    let site = Site::new(source, destination, Config { author: String::from("Carlos Galdino"), name: String::from("cg") });

    let walker                   = WalkDir::new(&site.source).into_iter();
    let mut templates: Templates = HashMap::new();
    let mut pages: Vec<Page>     = Vec::new();

    fs::remove_dir_all(&site.destination);

    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            match ext.to_str().unwrap() {
                "md" | "markdown" => pages.push(build_from_markdown(&entry, &site)),
                "html"            => pages.push(build_from_html(&entry, &site)),
                "mustache"        => add_template(&entry, &mut templates),
                _ => {
                    let path     = entry.path().to_str().unwrap().replace(&site.source, "");
                    let new_path = Path::new(&site.destination).join(path);
                    fs::create_dir_all(new_path.parent().unwrap());

                    fs::copy(entry.path(), new_path);
                }
            }
        }
    }

    let posts: Vec<Page> = pages.clone().into_iter().filter(|p| p.is_post()).collect();

    let site = Site { pages: pages, posts: posts, .. site };

    for page in &site.pages {
        render(&page, &site, &templates);
    }
}

fn add_template(entry: &DirEntry, templates: &mut Templates) {
    let file_name = entry.file_name().to_str().unwrap().replace(".mustache", "").to_string();
    let template = mustache::compile_path(entry.path());

    templates.insert(file_name, template.unwrap());
}

fn render(page: &Page, site: &Site, templates: &Templates) {
    match &*page.markup {
        "markdown" | "md" => render_markdown(&page, &site, &templates),
        _ => render_html(&page, &site, &templates),
    }
}

fn build_from_html(entry: &DirEntry, site: &Site) -> Page {
    let (attrs, content)   = utils::read_content(&entry);
    let path               = utils::new_path(&entry.path(), &site);

    Page::new(attrs, content, path, &site.config)
}

fn build_from_markdown(entry: &DirEntry, site: &Site) -> Page {
    let (attrs, content)   = utils::read_content(&entry);
    let rendered_content   = utils::render_markdown(content);
    let path               = utils::new_path(&entry.path(), &site);

    Page::new(attrs, rendered_content, path, &site.config)
}

fn render_markdown(page: &Page, site: &Site, templates: &Templates) {
    let mut file = utils::create_output_file(&page.path);
    let context  = Context { site: site, page: page };

    templates.get(&page.attributes.layout).unwrap().render(&mut file, &context);
}

fn render_html(page: &Page, site: &Site, templates: &Templates) {
    let page_template         = mustache::compile_str(&page.content);
    let mut rendered: Vec<u8> = Vec::new();
    let context               = Context { site: site, page: page };

    page_template.render(&mut rendered, &context);

    let page     = Page { content: String::from_utf8(rendered).unwrap(), .. page.clone() };
    let context  = Context { page: &page, site: site };
    let mut file = utils::create_output_file(&page.path);

    templates.get(&page.attributes.layout).unwrap().render(&mut file, &context);
}
