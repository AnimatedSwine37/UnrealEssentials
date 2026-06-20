pub mod assets;
pub mod metadata;
pub mod store;

use std::error::Error;

pub(crate) type GenericResult<T> = Result<T, Box<dyn Error>>;