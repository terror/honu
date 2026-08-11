use super::*;

#[test]
fn bash() {
  Test::new()
    .write(
      "history",
      indoc! {
        r#"
        #1700000000
        for foo in bar; do
          echo "$foo"
        done
        #1700000001
        cargo test
        "#
      },
    )
    .arguments(["import", "--path", "history", "bash"])
    .expected_stdout("imported 2 executions from history\n")
    .run()
    .inspect(|test| {
      assert_eq!(
        test.executions(),
        [
          Execution {
            command: indoc! {
              r#"
              for foo in bar; do
                echo "$foo"
              done
              "#
            }
            .trim_end()
            .into(),
            shell: Some("bash".into()),
            timestamp_ns: 1_700_000_000_000_000_000,
            ..Default::default()
          },
          Execution {
            command: "cargo test".into(),
            shell: Some("bash".into()),
            timestamp_ns: 1_700_000_001_000_000_000,
            ..Default::default()
          },
        ]
      );
    });
}

#[test]
fn defaults_are_shell_specific() {
  let test = Test::new()
    .write(".bash_history", "foo\n")
    .write(
      "fish/fish_history",
      indoc! {
        "
        - cmd: baz
          when: 2
        "
      },
    )
    .write(".zsh_history", ": 1:0;bar\n");

  let home = test.path("");

  test
    .env("HISTFILE", "")
    .env("HOME", &home)
    .arguments(["import", "zsh"])
    .expected_stdout("imported 1 execution from [ROOT]/.zsh_history\n")
    .run()
    .env("HISTFILE", "")
    .env("HOME", &home)
    .arguments(["import", "bash"])
    .expected_stdout("imported 1 execution from [ROOT]/.bash_history\n")
    .run()
    .env("HOME", &home)
    .arguments(["import", "fish"])
    .expected_stdout("imported 1 execution from [ROOT]/fish/fish_history\n")
    .run()
    .inspect(|test| {
      assert_eq!(
        test.executions(),
        [
          Execution {
            command: "foo".into(),
            shell: Some("bash".into()),
            timestamp_ns: 1,
            ..Default::default()
          },
          Execution {
            command: "bar".into(),
            duration_ns: Some(0),
            shell: Some("zsh".into()),
            timestamp_ns: 1_000_000_000,
            ..Default::default()
          },
          Execution {
            command: "baz".into(),
            shell: Some("fish".into()),
            timestamp_ns: 2_000_000_000,
            ..Default::default()
          },
        ]
      );
    });
}

#[test]
fn distinct_sources_are_tracked_independently() {
  Test::new()
    .write("foo", "bar\n")
    .write("baz", "bar\n")
    .arguments(["import", "--path", "foo", "zsh"])
    .expected_stdout("imported 1 execution from foo\n")
    .run()
    .arguments(["import", "--path", "baz", "zsh"])
    .expected_stdout("imported 1 execution from baz\n")
    .run()
    .arguments(["import", "--path", "foo", "zsh"])
    .expected_stdout("imported 0 executions from foo\n")
    .run()
    .arguments(["import", "--path", "baz", "zsh"])
    .expected_stdout("imported 0 executions from baz\n")
    .run()
    .inspect(|test| assert_eq!(test.executions().len(), 2));
}

#[test]
fn fish() {
  Test::new()
    .write(
      "history",
      indoc! {
        r"
        - cmd: git status
          when: 1700000000
        - cmd: for foo in bar\n    echo $foo\nend
          when: 1700000001
          paths:
            - /foo
        "
      },
    )
    .arguments(["import", "--path", "history", "fish"])
    .expected_stdout("imported 2 executions from history\n")
    .run()
    .inspect(|test| {
      assert_eq!(
        test.executions(),
        [
          Execution {
            command: "git status".into(),
            shell: Some("fish".into()),
            timestamp_ns: 1_700_000_000_000_000_000,
            ..Default::default()
          },
          Execution {
            command: indoc! {
              "
            for foo in bar
                echo $foo
            end
            "
            }
            .trim_end()
            .into(),
            shell: Some("fish".into()),
            timestamp_ns: 1_700_000_001_000_000_000,
            ..Default::default()
          },
        ]
      );
    });
}

#[test]
fn histfile_is_respected() {
  let test = Test::new()
    .write("bash-history", "foo\n")
    .write("zsh-history", ": 1:0;bar\n");

  let bash_history = test.path("bash-history");

  let zsh_history = test.path("zsh-history");

  test
    .env("HISTFILE", &bash_history)
    .arguments(["import", "bash"])
    .expected_stdout("imported 1 execution from [ROOT]/bash-history\n")
    .run()
    .env("HISTFILE", &zsh_history)
    .arguments(["import", "zsh"])
    .expected_stdout("imported 1 execution from [ROOT]/zsh-history\n")
    .run()
    .inspect(|test| {
      assert_eq!(
        test
          .executions()
          .iter()
          .map(|execution| execution.command.as_str())
          .collect::<Vec<_>>(),
        ["foo", "bar"],
      );
    });
}

#[test]
fn idempotent() {
  #[track_caller]
  fn case(shell: &str, history: &str) {
    Test::new()
      .write("history", history)
      .arguments(["import", "--path", "history", shell])
      .expected_stdout("imported 2 executions from history\n")
      .run()
      .arguments(["import", "--path", "history", shell])
      .expected_stdout("imported 0 executions from history\n")
      .run()
      .inspect(|test| assert_eq!(test.executions().len(), 2));
  }

  case("bash", "#1\nfoo\n#2\nbar\n");
  case("fish", "- cmd: foo\n  when: 1\n- cmd: bar\n  when: 2\n");
  case("zsh", ": 1:0;foo\n: 2:0;bar\n");
}

#[test]
fn parse_failure_does_not_partially_import() {
  Test::new()
    .write("history", "#1\nfoo\n#9223372037\nbar\n")
    .arguments(["import", "--path", "history", "bash"])
    .expected_stderr(indoc! {
      "
      error: failed to parse Bash history `history`

      because:
      - timestamp on history line 3 overflows nanoseconds
      "
    })
    .expected_status(1)
    .run()
    .inspect(|test| assert_eq!(test.executions().len(), 0))
    .write("history", "#1\nfoo\n#2\nbar\n")
    .arguments(["import", "--path", "history", "bash"])
    .expected_stdout("imported 2 executions from history\n")
    .run()
    .inspect(|test| assert_eq!(test.executions().len(), 2));
}

#[test]
fn path_takes_precedence_over_histfile() {
  let test = Test::new()
    .write("explicit", "foo\n")
    .write("history", "bar\n");

  let history = test.path("history");

  test
    .env("HISTFILE", &history)
    .arguments(["import", "--path", "explicit", "bash"])
    .expected_stdout("imported 1 execution from explicit\n")
    .run()
    .inspect(|test| {
      assert_eq!(test.executions()[0].command, "foo");
    });
}

#[test]
fn reconciles_insertions_with_existing_executions() {
  let test = Test::new()
    .write("history", "foo\nbar\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 2 executions from history\n")
    .run();

  let foo = test.execution_id("foo");
  let bar = test.execution_id("bar");

  let test = test
    .write("history", "baz\nfoo\nqux\nbar\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 2 executions from history\n")
    .run()
    .inspect(|test| {
      assert_eq!(
        test.executions(),
        [
          Execution {
            command: "baz".into(),
            shell: Some("zsh".into()),
            timestamp_ns: 1,
            ..Default::default()
          },
          Execution {
            command: "foo".into(),
            shell: Some("zsh".into()),
            timestamp_ns: 2,
            ..Default::default()
          },
          Execution {
            command: "qux".into(),
            shell: Some("zsh".into()),
            timestamp_ns: 3,
            ..Default::default()
          },
          Execution {
            command: "bar".into(),
            shell: Some("zsh".into()),
            timestamp_ns: 4,
            ..Default::default()
          },
        ]
      );
    });

  assert_eq!(test.execution_id("foo"), foo);
  assert_eq!(test.execution_id("bar"), bar);
}

#[test]
fn repeated_commands_are_reconciled_by_occurrence() {
  Test::new()
    .write("history", "foo\nfoo\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 2 executions from history\n")
    .run()
    .write("history", "foo\nfoo\nfoo\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 execution from history\n")
    .run()
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 0 executions from history\n")
    .run()
    .inspect(|test| assert_eq!(test.executions().len(), 3));
}

#[test]
fn truncated_records_are_retained() {
  let test = Test::new()
    .write("history", "foo\nbar\nbaz\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 3 executions from history\n")
    .run();

  let bar = test.execution_id("bar");
  let baz = test.execution_id("baz");

  let test = test
    .write("history", "bar\nbaz\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 0 executions from history\n")
    .run()
    .inspect(|test| assert_eq!(test.executions().len(), 3));

  assert_eq!(test.execution_id("bar"), bar);
  assert_eq!(test.execution_id("baz"), baz);
}

#[test]
fn zsh() {
  Test::new()
    .write(
      "history",
      indoc! {
        "
        git status
        : 1700000000:2;cargo test
        "
      },
    )
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 2 executions from history\n")
    .run()
    .inspect(|test| {
      assert_eq!(
        test.executions(),
        [
          Execution {
            command: "git status".into(),
            shell: Some("zsh".into()),
            timestamp_ns: 1,
            ..Default::default()
          },
          Execution {
            command: "cargo test".into(),
            duration_ns: Some(2_000_000_000),
            shell: Some("zsh".into()),
            timestamp_ns: 1_700_000_000_000_000_000,
            ..Default::default()
          },
        ]
      );
    });
}
