pub mod gzip;
pub mod handlebars;
pub mod traits;

pub struct Template<'a> {
    pub language: &'a str,
    pub content: &'a str,
}
