use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Execution {
  pub command: String,
  pub directory: Option<PathBuf>,
  pub duration_ns: Option<i64>,
  pub exit_code: Option<i32>,
  pub hostname: Option<String>,
  pub session: Option<String>,
  pub shell: Option<String>,
  pub timestamp_ns: i64,
}

impl Execution {
  /// Returns the execution directory as UTF-8.
  ///
  /// # Errors
  ///
  /// Returns an error if the directory is not valid UTF-8.
  pub fn directory(&self) -> Result<Option<&str>> {
    self
      .directory
      .as_deref()
      .map(|directory| {
        directory.to_str().ok_or(Error::InvalidExecutionDirectory)
      })
      .transpose()
  }
}

impl FromRow for Execution {
  fn from_row(row: &Row<'_>) -> Result<Self> {
    Ok(Self {
      command: row.get("command")?,
      directory: row
        .get::<_, Option<String>>("directory")?
        .map(PathBuf::from),
      duration_ns: row.get("duration_ns")?,
      exit_code: row.get("exit_code")?,
      hostname: row.get("hostname")?,
      session: row.get("session")?,
      shell: row.get("shell")?,
      timestamp_ns: row.get("timestamp_ns")?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn directory() {
    assert_eq!(Execution::default().directory().unwrap(), None);

    assert_eq!(
      Execution {
        directory: Some("foo".into()),
        ..Default::default()
      }
      .directory()
      .unwrap(),
      Some("foo"),
    );
  }
}
