use super::*;

#[derive(Debug, Clap)]
pub(crate) struct Init {
  shell: Shell,
}

impl Init {
  pub(crate) fn run(self) {
    print!("{}", self.shell.init());
  }
}
