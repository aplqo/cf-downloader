use std::{boxed::Box, error::Error as StdError, result::Result as StdResult};

pub(crate) type BoxedError = Box<dyn StdError + Send>;

pub type Result<T> = StdResult<T, BoxedError>;
