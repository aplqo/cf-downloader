pub mod gzip;
pub mod handlebars {
    pub mod encode;
    pub mod error;
    pub mod meta;
}
pub mod traits;

pub struct Template {
    pub language: String,
    pub content: String,
}
