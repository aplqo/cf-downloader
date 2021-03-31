use std::{boxed::Box, error::Error as StdError, result::Result as StdResult};

pub trait Error: StdError + Send {}
pub(crate) type BoxedError = Box<dyn StdError + Send>;

pub type Result<T> = StdResult<T, BoxedError>;
