use super::*;

#[derive(Debug, Clap)]
pub(crate) struct Import {
  #[arg(long, value_name = "PATH")]
  path: Option<PathBuf>,
  shell: Option<Shell>,
}

impl Import {
  pub(crate) fn run(self) -> Result {
    let config = config::Config::load()?;

    self
      .shell
      .or(config.import.shell)
      .map_or_else(Shell::detect, Ok)?
      .import(&Database::load()?, self.path.as_deref())
  }
}
