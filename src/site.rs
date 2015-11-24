use std::fs;
use walkdir::{ DirEntry, WalkDir };
use page::{ Page, Markup };
use std::collections::HashMap;
use mustache;
use utils;

type Templates = HashMap<String, mustache::Template>;

#[derive(RustcEncodable)]
pub struct Config {
    pub author:      String,
    pub name:        String,
    pub source:      String,
    pub destination: String,
}

#[derive(RustcEncodable)]
struct Context<'a> {
    site: &'a Site,
    page: &'a Page
}

#[derive(RustcEncodable)]
pub struct Site {
    pub config: Config,
    pages: Vec<Page>,
    posts: Vec<Page>,
}

impl Site {
    fn new(config: Config) -> Site {
        Site { config: config, pages: Vec::new(), posts: Vec::new() }
    }
}

pub fn build(source: String, destination: String) {
    let site = Site::new(Config { author: String::from("Carlos Galdino"), name: String::from("cg"), source: source.clone(), destination: destination.clone() });

    let walker                   = WalkDir::new(source).into_iter();
    let mut templates: Templates = HashMap::new();
    let mut pages: Vec<Page>     = Vec::new();

    fs::remove_dir_all(destination);

    for entry in walker.filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            // TODO: use `extract_markup`
            match ext.to_str().unwrap() {
                "md" | "markdown" => pages.push(build_from_markdown(&entry, &site.config)),
                "html"            => pages.push(build_from_html(&entry, &site.config)),
                "mustache"        => add_template(&entry, &mut templates),
                _                 => utils::copy_file(&entry, &site.config),
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
    match page.markup {
        Markup::Markdown => render_markdown(&page, &site, &templates),
        Markup::HTML     => render_html(&page, &site, &templates),
    }
}

fn build_from_html(entry: &DirEntry, config: &Config) -> Page {
    let (attrs, content)   = utils::read_content(&entry);

    Page::new(attrs, content, entry.path(), &config)
}

fn build_from_markdown(entry: &DirEntry, config: &Config) -> Page {
    let (attrs, content)   = utils::read_content(&entry);
    let rendered_content   = utils::render_markdown(content);

    Page::new(attrs, rendered_content, entry.path(), &config)
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
