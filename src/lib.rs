use std::path::PathBuf;

pub use {error::Error, execution::Execution};

mod error;
mod execution;

type Result<T = ()> = std::result::Result<T, Error>;
