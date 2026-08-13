use super::*;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Config {
  pub(crate) import: Import,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Import {
  pub(crate) shell: Option<Shell>,
}

impl Config {
  pub(crate) fn load() -> Result<Self> {
    confy::load("honu", "config").context("failed to load configuration")
  }
}
