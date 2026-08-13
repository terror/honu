use {
  super::*, add::Add, backup::Backup, r#import::Import, init::Init, list::List,
  search::Search,
};

mod add;
mod backup;
mod clear;
mod r#import;
mod init;
mod list;
mod search;

#[derive(Debug, Clap)]
pub(crate) enum Subcommand {
  #[command(about = "Record a shell command", alias = "a")]
  Add(Add),
  #[command(about = "Back up the database")]
  Backup(Backup),
  #[command(about = "Clear the history database")]
  Clear,
  #[command(about = "Import shell history", alias = "i")]
  Import(Import),
  #[command(about = "Generate shell integration")]
  Init(Init),
  #[command(about = "List recent shell commands", alias = "l")]
  List(List),
  #[command(about = "Search shell commands", alias = "s")]
  Search(Search),
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Add(add) => add.run(),
      Self::Backup(backup) => backup.run(),
      Self::Clear => clear::run(),
      Self::Import(import) => import.run(),
      Self::Init(init) => {
        init.run();
        Ok(())
      }
      Self::List(list) => list.run(),
      Self::Search(search) => search.run(),
    }
  }
}
