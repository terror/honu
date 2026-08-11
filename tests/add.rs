use super::*;

#[test]
fn defaults() {
  let test = Test::new();

  let directory = test.path("").canonicalize().unwrap();

  test
    .arguments(["add", "--timestamp-ns", "1", "--", "foo"])
    .run()
    .inspect(|test| {
      assert_eq!(
        test.executions(),
        [Execution {
          command: "foo".into(),
          directory: Some(directory),
          timestamp_ns: 1,
          ..Default::default()
        }],
      );
    });
}

#[test]
fn record() {
  Test::new()
    .arguments([
      "add",
      "--directory",
      "/foo",
      "--duration-ns",
      "2",
      "--exit-code",
      "0",
      "--hostname",
      "foo",
      "--session",
      "bar",
      "--shell",
      "zsh",
      "--timestamp-ns",
      "1",
      "--",
      "foo",
    ])
    .run()
    .inspect(|test| {
      assert_eq!(
        test.executions(),
        [Execution {
          command: "foo".into(),
          directory: Some("/foo".into()),
          duration_ns: Some(2),
          exit_code: Some(0),
          hostname: Some("foo".into()),
          session: Some("bar".into()),
          shell: Some("zsh".into()),
          timestamp_ns: 1,
        }],
      );
    });
}
