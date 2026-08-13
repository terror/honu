use super::*;

#[derive(Debug, Clap)]
pub(crate) struct Import {
  #[arg(long, value_name = "PATH")]
  path: Option<PathBuf>,
  shell: Option<Shell>,
}

impl Import {
  pub(crate) fn run(
    self,
    database: &Database,
    configured_shell: Option<Shell>,
  ) -> Result {
    self
      .shell
      .or(configured_shell)
      .map_or_else(Shell::detect, Ok)?
      .import(database, self.path.as_deref())
  }
}
