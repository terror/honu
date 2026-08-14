use super::*;

#[derive(Debug, Clap)]
pub(crate) struct Backup {
  #[arg(long)]
  force: bool,
  #[arg(value_name = "PATH")]
  path: PathBuf,
}

impl Backup {
  pub(crate) fn run(self) -> Result {
    let database = Database::load()?;

    let parent = self
      .path
      .parent()
      .filter(|parent| !parent.as_os_str().is_empty())
      .unwrap_or_else(|| Path::new("."));

    fs::create_dir_all(parent)?;

    if !self.force && self.path.try_exists()? {
      bail!(
        "backup `{}` already exists; use `--force` to overwrite it",
        self.path.display(),
      );
    }

    let temporary = NamedTempFile::new_in(parent)?;

    database.backup(temporary.path())?;

    #[cfg(unix)]
    fs::set_permissions(temporary.path(), fs::Permissions::from_mode(0o600))?;

    let result = if self.force {
      temporary.persist(&self.path)
    } else {
      temporary.persist_noclobber(&self.path)
    };

    if let Err(error) = result {
      if !self.force && error.error.kind() == io::ErrorKind::AlreadyExists {
        bail!(
          "backup `{}` already exists; use `--force` to overwrite it",
          self.path.display(),
        );
      }

      return Err(error.error.into());
    }

    Ok(())
  }
}
