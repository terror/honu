use {
  anyhow::{Context, Error, bail},
  arguments::Arguments,
  choice::Choice,
  clap::{Parser as Clap, ValueEnum},
  command::Command,
  config::Config,
  count::Count,
  database::Database,
  honu::Execution,
  imara_diff::{Algorithm, Diff, InternedInput},
  indicatif::{ProgressBar, ProgressStyle},
  line::Line,
  lines::Lines,
  parser::Parser,
  progress::Progress,
  ratatui::{
    style::{Color, Modifier},
    text::Span,
  },
  record::Record,
  records::Records,
  rusqlite::{Connection, MAIN_DB, Transaction, TransactionBehavior, params},
  serde::{Deserialize, Serialize},
  shell::Shell,
  skim::{
    CaseMatching, DisplayContext, Skim, SkimItem, SkimItemSender,
    options::SkimOptionsBuilder, prelude::bounded,
  },
  std::{
    borrow::Cow,
    env, fmt,
    fmt::{Display, Formatter},
    fs,
    io::{self, BufRead, BufReader, IsTerminal, Read},
    mem,
    num::NonZeroU8,
    path::{Path, PathBuf},
    process, str,
    sync::Arc,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
  },
  str_ext::StrExt,
  subcommand::Subcommand,
  tally::Tally,
  tempfile::NamedTempFile,
  unicode_segmentation::UnicodeSegmentation,
  uuid::Uuid,
};

#[cfg(unix)]
use {std::os::unix::fs::PermissionsExt, xdg::BaseDirectories};

mod arguments;
mod choice;
mod command;
mod config;
mod count;
mod database;
mod line;
mod lines;
mod parser;
mod progress;
mod record;
mod records;
mod shell;
mod str_ext;
mod subcommand;
mod tally;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error}");

    for (i, error) in error.chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {error}");
    }

    process::exit(1);
  }
}
