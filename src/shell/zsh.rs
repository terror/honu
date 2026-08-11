use super::*;

pub(super) use {metafied::Metafied, parser::ZshParser};

mod metafied;
mod parser;

pub(super) const DEFAULT_HISTORY_FILE: &str = ".zsh_history";
pub(super) const FORMAT: &str = "zsh";
pub(super) const INIT: &str = include_str!("zsh/init.zsh");
pub(super) const NAME: &str = "Zsh";
