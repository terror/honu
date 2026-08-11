use super::*;

#[test]
fn clear() {
  let test = Test::new()
    .write("history", "foo\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 executions from history\n")
    .run();

  let id = test.execution_id("foo");

  let test = test
    .argument("clear")
    .run()
    .assert_executions([])
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 executions from history\n")
    .run()
    .assert_execution_count(1);

  assert_ne!(test.execution_id("foo"), id);
}
