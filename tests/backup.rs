use super::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn backup() {
  let test = Test::new()
    .write("history", "foo\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 execution from history\n")
    .run()
    .arguments(["backup", "foo/bar/honu.sqlite"])
    .run()
    .inspect(|test| {
      let database = test.database_at("foo/bar/honu.sqlite");

      assert_eq!(
        database
          .query_row("SELECT COUNT(*) FROM executions", [], |row| {
            row.get::<_, i64>(0)
          })
          .unwrap(),
        1,
      );
    });

  #[cfg(unix)]
  assert_eq!(
    test
      .path("foo/bar/honu.sqlite")
      .metadata()
      .unwrap()
      .permissions()
      .mode()
      & 0o777,
    0o600,
  );

  test
    .arguments(["backup", "foo/bar/honu.sqlite"])
    .expected_stderr(
      "error: backup `foo/bar/honu.sqlite` already exists; use `--force` to overwrite it\n",
    )
    .expected_status(1).run()
    .write(
      "history",
      indoc! {
        "
        foo
        bar
        "
      },
    )
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 execution from history\n")
    .run()
    .arguments(["backup", "--force", "foo/bar/honu.sqlite"])
    .run()
    .inspect(|test| {
      let database = test.database_at("foo/bar/honu.sqlite");

      assert_eq!(
        database
          .query_row("SELECT COUNT(*) FROM executions", [], |row| {
            row.get::<_, i64>(0)
          })
          .unwrap(),
        2,
      );
    });
}

#[test]
fn backup_is_usable_application_database() {
  let test = Test::new()
    .write("history", "foo\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 execution from history\n")
    .run()
    .arguments(["backup", "backup/honu/history.db"])
    .run();

  let backup = test.path("backup");

  let test = test
    .env("XDG_DATA_HOME", &backup)
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 0 executions from history\n")
    .run();

  fs::OpenOptions::new()
    .append(true)
    .open(test.path("history"))
    .unwrap()
    .write_all(b"bar\n")
    .unwrap();

  test
    .env("XDG_DATA_HOME", backup)
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 execution from history\n")
    .run()
    .inspect(|test| assert_eq!(test.executions().len(), 1))
    .inspect(|test| {
      let database = test.database_at("backup/honu/history.db");

      assert_eq!(
        database
          .query_row("SELECT COUNT(*) FROM executions", [], |row| {
            row.get::<_, i64>(0)
          })
          .unwrap(),
        2,
      );
    });
}
