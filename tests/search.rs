use super::*;

#[test]
fn empty() {
  Test::new().arguments(["search", "--", "foo"]).run();
}

#[test]
fn non_interactive() {
  Test::new()
    .arguments(["add", "--timestamp-ns", "1", "--", "foo"])
    .run()
    .arguments(["add", "--timestamp-ns", "2", "--", "bar"])
    .run()
    .arguments(["search", "--", "foo"])
    .expected_stdout("foo\n")
    .run();
}
