use super::*;

pub(super) mod metafied;
pub(super) mod parser;

pub(super) const DEFAULT_HISTORY_FILE: &str = ".zsh_history";
pub(super) const FORMAT: &str = "zsh";
pub(super) const INIT: &str = include_str!("zsh/init.zsh");
pub(super) const NAME: &str = "Zsh";
