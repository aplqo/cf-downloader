pub mod gzip;
pub mod handlebars {
    pub mod encode;
    mod error;
    pub mod meta;

    pub use error::{Error, Result};
}
mod traits;

pub use traits::{DataDecoder, DataEncoder, MetaEncoding};

pub struct Template {
    pub language: String,
    pub content: String,
}
