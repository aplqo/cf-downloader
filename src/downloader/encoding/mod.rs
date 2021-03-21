pub mod gzip;
pub mod handlebars;
pub mod traits;
mod utility;

pub struct Template {
    pub language: String,
    pub content: String,
}
