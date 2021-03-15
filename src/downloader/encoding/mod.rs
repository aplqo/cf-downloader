pub mod gzip;
pub mod handlebars;
pub mod traits;

pub struct Template {
    pub language: String,
    pub content: String,
}
