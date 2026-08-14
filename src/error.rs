#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("execution directory is not valid UTF-8")]
  InvalidExecutionDirectory,
  #[error(transparent)]
  Sqlite(#[from] rusqlite::Error),
}
