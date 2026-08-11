use super::*;

#[test]
fn clear() {
  let test = Test::new()
    .write("history", "foo\n")
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 execution from history\n")
    .run();

  let id = test.execution_id("foo");

  let test = test
    .argument("clear")
    .run()
    .inspect(|test| assert!(test.executions().is_empty()))
    .arguments(["import", "--path", "history", "zsh"])
    .expected_stdout("imported 1 execution from history\n")
    .run()
    .inspect(|test| assert_eq!(test.executions().len(), 1));

  assert_ne!(test.execution_id("foo"), id);
}
