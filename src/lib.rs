use {
  rusqlite::Row,
  std::{borrow::Cow, path::PathBuf},
};

pub use {command::Command, error::Error, execution::Execution};

mod command;
mod error;
mod execution;

type Result<T = ()> = std::result::Result<T, Error>;
