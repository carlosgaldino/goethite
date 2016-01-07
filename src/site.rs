use std::fs;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use page::Page;
use std::collections::HashMap;
use mustache;
use utils::{self, Markup};
use config::Config;
use error::{GoethiteError, Result};

type Templates = HashMap<String, mustache::Template>;

#[derive(RustcEncodable)]
struct Context<'a> {
    site: &'a Site,
    page: &'a Page,
}

#[derive(RustcEncodable)]
pub struct Site {
    pub config: Config,
    pages: Vec<Page>,
    posts: Vec<Page>,
}

impl Site {
    fn new(config: Config) -> Site {
        Site {
            config: config,
            pages: Vec::new(),
            posts: Vec::new(),
        }
    }
}

pub fn build(source: String, destination: String) -> Result<()> {
    let config = try!(Config::new(source.clone(), destination.clone()));
    let site = Site::new(config);

    let walker = WalkDir::new(source).into_iter();
    let mut templates: Templates = HashMap::new();
    let mut pages: Vec<Page> = Vec::new();

    fs::remove_dir_all(destination);

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = try!(entry);

        match utils::extract_markup(entry.path()) {
            Some(m) => {
                match m {
                    Markup::Markdown => pages.push(try!(build_from_markdown(&entry, &site.config))),
                    Markup::HTML => pages.push(try!(build_from_html(&entry, &site.config))),
                    Markup::Mustache => try!(add_template(&entry, &mut templates)),
                }
            }
            None => {
                if entry.file_type().is_file() {
                    try!(utils::copy_file(&entry, &site.config));
                }
            }
        }
    }

    let mut posts: Vec<Page> = pages.clone().into_iter().filter(|p| p.is_post()).collect();
    posts.sort_by(|a, b| b.attributes.date.cmp(&a.attributes.date));

    let site = Site {
        pages: pages,
        posts: posts,
        ..site
    };

    for page in &site.pages {
        match render(&page, &site, &templates) {
            Ok(_) => {}
            Err(err) => {
                println!("Page '{}' not rendered because: {}",
                         page.attributes.title,
                         err)
            }
        }
    }

    Ok(())
}

fn add_template(entry: &DirEntry, templates: &mut Templates) -> Result<()> {
    let file_name = try!(entry.file_name().to_str().ok_or(GoethiteError::Other))
                        .replace(".mustache", "")
                        .to_string();
    let template = try!(mustache::compile_path(entry.path()));

    templates.insert(file_name, template);

    Ok(())
}

fn render(page: &Page, site: &Site, templates: &Templates) -> Result<()> {
    match page.markup {
        Markup::Markdown => render_markdown(&page, &site, &templates),
        Markup::HTML => render_html(&page, &site, &templates),
        _ => Ok(()),
    }
}

fn build_from_html(entry: &DirEntry, config: &Config) -> Result<Page> {
    let content = try!(utils::read_content(&entry));

    Page::new(content.attributes, content.text, entry.path(), &config)
}

fn build_from_markdown(entry: &DirEntry, config: &Config) -> Result<Page> {
    let content = try!(utils::read_content(&entry));
    let rendered_content = utils::render_markdown(content.text);

    Page::new(content.attributes, rendered_content, entry.path(), &config)
}

fn render_markdown(page: &Page, site: &Site, templates: &Templates) -> Result<()> {
    let template = match templates.get(&page.attributes.layout) {
        Some(t) => t,
        None => return Err(GoethiteError::MissingLayout(page.attributes.layout.to_owned())),
    };

    let mut file = try!(utils::create_output_file(&page.path));
    let context = Context {
        site: site,
        page: page,
    };

    try!(template.render(&mut file, &context));

    Ok(())
}

fn render_html(page: &Page, site: &Site, templates: &Templates) -> Result<()> {
    let template = match templates.get(&page.attributes.layout) {
        Some(t) => t,
        None => return Err(GoethiteError::MissingLayout(page.attributes.layout.to_owned())),
    };

    let page_template = mustache::compile_str(&page.content);
    let mut rendered: Vec<u8> = Vec::new();
    let context = Context {
        site: site,
        page: page,
    };

    try!(page_template.render(&mut rendered, &context));

    let page = Page { content: String::from_utf8(rendered).unwrap(), ..page.clone() };
    let context = Context {
        page: &page,
        site: site,
    };
    let mut file = try!(utils::create_output_file(&page.path));

    try!(template.render(&mut file, &context));

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
}
