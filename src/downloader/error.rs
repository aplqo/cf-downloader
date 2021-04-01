use std::{boxed::Box, error::Error as StdError, result::Result as StdResult};

pub trait Error: StdError + Send + 'static {}
pub(crate) type BoxedError = Box<dyn Error>;

pub type Result<T> = StdResult<T, BoxedError>;

impl<E: StdError + Send + 'static> Error for E {}
