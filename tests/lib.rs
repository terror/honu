use {
  anyhow::Context,
  honu::Execution,
  indoc::indoc,
  rusqlite::Connection,
  std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    io::Write,
    iter::once,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str,
  },
  tempfile::TempDir,
};

#[macro_use]
extern crate pretty_assertions;

mod add;
mod backup;
mod clear;
mod import;
mod init;
mod list;
mod search;
mod shell;

#[derive(Debug)]
struct Test {
  arguments: Vec<OsString>,
  environments: Vec<(OsString, OsString)>,
  executable: OsString,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: String,
  stdin: Option<Vec<u8>>,
  tempdir: TempDir,
}

impl Test {
  fn argument(self, argument: impl AsRef<OsStr>) -> Self {
    self.arguments([argument])
  }

  fn arguments<I, S>(mut self, arguments: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    assert!(self.arguments.is_empty());

    self.arguments.extend(
      arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned()),
    );

    self
  }

  fn config_path(&self) -> PathBuf {
    #[cfg(not(windows))]
    let path = "honu/config.toml";

    #[cfg(windows)]
    let path = "honu/config/config.toml";

    self.path(path)
  }

  fn database(&self) -> Connection {
    self.database_at("honu/history.db")
  }

  fn database_at(&self, path: impl AsRef<Path>) -> Connection {
    Connection::open(self.path(path)).unwrap()
  }

  fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
    self
      .environments
      .push((key.as_ref().to_owned(), value.as_ref().to_owned()));

    self
  }

  fn execution_id(&self, command: &str) -> String {
    self
      .database()
      .query_row(
        "SELECT id FROM executions WHERE command = ?1",
        [command],
        |row| row.get(0),
      )
      .unwrap()
  }

  fn executions(&self) -> Vec<Execution> {
    let connection = self.database();

    let mut statement = connection
      .prepare(
        "SELECT
           command,
           timestamp_ns,
           duration_ns,
           exit_code,
           directory,
           session,
           hostname,
           shell
         FROM executions
         ORDER BY timestamp_ns, id",
      )
      .unwrap();

    statement
      .query_map([], |row| {
        Ok(Execution {
          command: row.get(0)?,
          timestamp_ns: row.get(1)?,
          duration_ns: row.get(2)?,
          exit_code: row.get(3)?,
          directory: row
            .get::<_, Option<String>>(4)?
            .map(PathBuf::from)
            .map(|directory| directory.canonicalize().unwrap_or(directory)),
          session: row.get(5)?,
          hostname: row.get(6)?,
          shell: row.get(7)?,
        })
      })
      .unwrap()
      .collect::<rusqlite::Result<Vec<_>>>()
      .unwrap()
  }

  fn expected_status(mut self, expected_status: i32) -> Self {
    self.expected_status = expected_status;
    self
  }

  fn expected_stderr(mut self, expected_stderr: &str) -> Self {
    assert!(self.expected_stderr.is_empty());
    self.expected_stderr = expected_stderr.into();
    self
  }

  fn expected_stdout(mut self, expected_stdout: &str) -> Self {
    assert!(self.expected_stdout.is_empty());
    self.expected_stdout = expected_stdout.into();
    self
  }

  fn inspect(self, inspect: impl FnOnce(&Self)) -> Self {
    inspect(&self);
    self
  }

  fn new() -> Self {
    Self::with_tempdir(TempDir::with_prefix("honu-test").unwrap())
  }

  fn path(&self, path: impl AsRef<Path>) -> PathBuf {
    self.tempdir.path().join(path)
  }

  fn program(mut self, executable: impl AsRef<OsStr>) -> Self {
    executable.as_ref().clone_into(&mut self.executable);
    self
  }

  #[track_caller]
  fn run(self) -> Self {
    let mut command = Command::new(&self.executable);

    let path = env::join_paths(
      once(
        Path::new(env!("CARGO_BIN_EXE_honu"))
          .parent()
          .unwrap()
          .to_path_buf(),
      )
      .chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())),
    )
    .unwrap();

    command
      .current_dir(self.tempdir.path())
      .env("APPDATA", self.tempdir.path())
      .env("HOME", self.tempdir.path())
      .env("XDG_CONFIG_HOME", self.tempdir.path())
      .env("XDG_DATA_HOME", self.tempdir.path())
      .env("ZDOTDIR", self.tempdir.path())
      .env("PATH", path)
      .args(&self.arguments)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());

    for (key, value) in &self.environments {
      command.env(key, value);
    }

    let mut child = command
      .spawn()
      .with_context(|| {
        format!("failed to run `{}`", Path::new(&self.executable).display())
      })
      .unwrap();

    if let Some(stdin) = &self.stdin {
      child.stdin.take().unwrap().write_all(stdin).unwrap();
    }

    let output = child.wait_with_output().unwrap();

    let normalize = |text: &str| {
      text
        .replace(&self.tempdir.path().display().to_string(), "[ROOT]")
        .replace('\\', "/")
    };

    let stderr = normalize(str::from_utf8(&output.stderr).unwrap());

    assert_eq!(
      output.status.code(),
      Some(self.expected_status),
      "unexpected exit status\nstderr: {stderr}",
    );

    assert_eq!(stderr, self.expected_stderr);

    let stdout = normalize(str::from_utf8(&output.stdout).unwrap());

    assert_eq!(stdout, self.expected_stdout);

    Self::with_tempdir(self.tempdir)
  }

  fn stdin(mut self, stdin: impl AsRef<[u8]>) -> Self {
    assert!(self.stdin.is_none());
    self.stdin = Some(stdin.as_ref().into());
    self
  }

  fn with_tempdir(tempdir: TempDir) -> Self {
    Self {
      arguments: Vec::new(),
      environments: Vec::new(),
      executable: env!("CARGO_BIN_EXE_honu").into(),
      expected_status: 0,
      expected_stderr: String::new(),
      expected_stdout: String::new(),
      stdin: None,
      tempdir,
    }
  }

  fn write(self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Self {
    let path = self.path(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
    self
  }

  fn write_config(self, contents: impl AsRef<[u8]>) -> Self {
    let path = self.config_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
    self
  }
}
