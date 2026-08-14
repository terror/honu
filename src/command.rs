use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Command {
  pub directory: Option<PathBuf>,
  pub exit_code: Option<i32>,
  pub text: String,
  pub timestamp_ns: i64,
}

impl Command {
  #[must_use]
  pub fn directory_name(&self) -> Option<Cow<'_, str>> {
    self.directory.as_deref().map(|directory| {
      directory
        .file_name()
        .unwrap_or(directory.as_os_str())
        .to_string_lossy()
    })
  }
}

impl TryFrom<&Row<'_>> for Command {
  type Error = Error;

  fn try_from(row: &Row<'_>) -> Result<Self> {
    Ok(Self {
      directory: row
        .get::<_, Option<String>>("directory")?
        .map(PathBuf::from),
      exit_code: row.get("exit_code")?,
      text: row.get("text")?,
      timestamp_ns: row.get("timestamp_ns")?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn directory_name() {
    #[track_caller]
    fn case(directory: Option<&str>, expected: Option<&str>) {
      assert_eq!(
        Command {
          directory: directory.map(PathBuf::from),
          ..Default::default()
        }
        .directory_name()
        .as_deref(),
        expected,
      );
    }

    case(None, None);
    case(Some("foo/bar"), Some("bar"));
    case(Some("/"), Some("/"));
  }
}
