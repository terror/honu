use super::*;

#[test]
fn empty() {
  Test::new().arguments(["search", "--", "foo"]).run();
}
