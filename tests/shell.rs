use super::*;

#[test]
#[ignore = "requires bash"]
fn bash_init() {
  Test::new()
    .program("bash")
    .argument("-n")
    .stdin(include_str!("../src/shell/bash/init.bash"))
    .success();
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
#[ignore = "requires fish"]
fn fish_init() {
  Test::new()
    .program("fish")
    .argument("-n")
    .stdin(include_str!("../src/shell/fish/init.fish"))
    .success();
}

#[test]
#[ignore = "requires fish"]
fn fish_private_mode_is_not_recorded() {
  Test::new()
    .program("fish")
    .argument("--no-config")
    .stdin(indoc! {
      "
      set -g fish_private_mode 1
      honu init fish | source
      emit fish_preexec true
      true
      emit fish_postexec
      "
    })
    .success()
    .assert_execution_count(0);
}

#[test]
#[ignore = "requires fish"]
fn fish_records_execution() {
  Test::new()
    .program("fish")
    .argument("--no-config")
    .stdin(indoc! {
      "
      set -e fish_private_mode
      honu init fish | source
      emit fish_preexec true
      true
      emit fish_postexec
      emit fish_preexec false
      false
      emit fish_postexec
      "
    })
    .success()
    .assert_recorded(&[("true", 0, "fish"), ("false", 1, "fish")]);
}

#[test]
#[ignore = "requires zsh"]
fn zsh_init() {
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
