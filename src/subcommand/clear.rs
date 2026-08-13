use super::*;

pub(crate) fn run() -> Result {
  Database::load()?.clear()
}
