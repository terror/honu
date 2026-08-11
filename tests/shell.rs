use super::*;

#[test]
#[ignore = "requires bash"]
fn bash_records_execution() {
  Test::new()
    .program("bash")
    .arguments([
      "-c",
      "exec bash --noprofile --rcfile .bashrc -i 2>/dev/null",
    ])
    .write(
      ".bashrc",
      indoc! {
        r#"
        PS1=
        PS2=
        eval "$(honu init bash)"
        "#
      },
    )
    .stdin("true\nfalse\nexit\n")
    .status(1)
    .assert_recorded(&[("true", 0, "bash"), ("false", 1, "bash")]);
}

#[test]
#[ignore = "requires bash"]
fn bash_preserves_scalar_prompt_command() {
  Test::new()
    .program("bash")
    .arguments([
      "-c",
      "exec bash --noprofile --rcfile .bashrc -i 2>/dev/null",
    ])
    .write(
      ".bashrc",
      indoc! {
        r#"
        PROMPT_COMMAND='printf "%s\n" foo;'
        PS1=
        PS2=
        eval "$(honu init bash)"
        "#
      },
    )
    .stdin("true\nexit\n")
    .stdout("foo\nfoo\n")
    .success()
    .assert_recorded(&[("true", 0, "bash")]);
}

fn fish() -> Test {
  let command = if cfg!(target_os = "linux") {
    "exec script -q -e -c 'fish --interactive' /dev/null >/dev/null 2>&1"
  } else {
    "exec script -q -e /dev/null fish --interactive >/dev/null 2>&1"
  };

  Test::new().program("sh").arguments(["-c", command])
}

#[test]
#[ignore = "requires fish"]
fn fish_records_execution() {
  fish()
    .write(
      "fish/config.fish",
      indoc! {
        "
        function fish_prompt
        end
        honu init fish | source
        "
      },
    )
    .stdin("true\nfalse\nexit\n")
    .status(1)
    .assert_recorded(&[("true", 0, "fish"), ("false", 1, "fish")]);
}

#[test]
#[ignore = "requires fish"]
fn fish_private_mode_is_not_recorded() {
  fish()
    .write(
      "fish/config.fish",
      indoc! {
        "
        function fish_prompt
        end
        set -g fish_private_mode 1
        honu init fish | source
        "
      },
    )
    .stdin("true\nexit\n")
    .success()
    .assert_execution_count(0);
}

#[test]
#[ignore = "requires bash"]
fn init_bash() {
  Test::new()
    .program("bash")
    .argument("-n")
    .stdin(include_str!("../src/shell/bash/init.bash"))
    .success();
}

#[test]
#[ignore = "requires fish"]
fn init_fish() {
  Test::new()
    .program("fish")
    .argument("-n")
    .stdin(include_str!("../src/shell/fish/init.fish"))
    .success();
}

#[test]
#[ignore = "requires zsh"]
fn init_zsh() {
  Test::new()
    .program("zsh")
    .argument("-n")
    .stdin(include_str!("../src/shell/zsh/init.zsh"))
    .success();
}

#[test]
#[ignore = "requires zsh"]
fn zsh_records_execution() {
  Test::new()
    .program("zsh")
    .arguments(["-d", "-i"])
    .write(
      ".zshrc",
      indoc! {
        r#"
        exec 2>/dev/null
        PROMPT=
        RPROMPT=
        eval "$(honu init zsh)"
        "#
      },
    )
    .stdin("true\nfalse\nexit\n")
    .status(1)
    .assert_recorded(&[("true", 0, "zsh"), ("false", 1, "zsh")]);
}
