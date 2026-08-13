use super::*;

#[test]
fn shells() {
  #[track_caller]
  fn case(shell: &str, expected: &str) {
    Test::new()
      .arguments(["init", shell])
      .expected_stdout(&expected.replace('\\', "/"))
      .run()
      .inspect(|test| {
        assert!(!test.config_path().exists());
        assert!(!test.path("honu/history.db").exists());
      });
  }

  case("bash", include_str!("../src/shell/bash/init.bash"));
  case("fish", include_str!("../src/shell/fish/init.fish"));
  case("zsh", include_str!("../src/shell/zsh/init.zsh"));
}
