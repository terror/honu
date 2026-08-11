use super::*;

#[test]
fn list() {
  Test::new()
    .arguments([
      "add",
      "--timestamp-ns",
      "1",
      "--exit-code",
      "1",
      "--",
      "foo",
    ])
    .run()
    .arguments([
      "add",
      "--timestamp-ns",
      "2",
      "--exit-code",
      "0",
      "--",
      "bar",
    ])
    .run()
    .argument("list")
    .expected_stdout(indoc! {
      "
      2\t0\tbar
      1\t1\tfoo
      "
    })
    .run()
    .arguments(["list", "--limit", "1"])
    .expected_stdout("2\t0\tbar\n")
    .run()
    .arguments(["list", "--limit", "0"])
    .run();
}
