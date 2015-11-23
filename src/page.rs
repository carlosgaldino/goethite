use std::path::PathBuf;

#[derive(RustcEncodable)]
pub struct Page {
    pub content: String,
    pub path: PathBuf,
}
