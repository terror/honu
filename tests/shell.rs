use super::*;

#[test]
#[ignore = "requires bash"]
fn bash_init() {
  Test::new()
    .program("bash")
    .argument("-n")
    .stdin(include_str!("../src/shell/bash/init.bash"))
    .run();
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
    .expected_stdout("foo\nfoo\n")
    .run()
    .inspect(|test| {
      assert_eq!(
        test
          .executions()
          .into_iter()
          .map(|execution| {
            (execution.command, execution.exit_code, execution.shell)
          })
          .collect::<Vec<_>>(),
        [("true".into(), Some(0), Some("bash".into()))],
      );
    });
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
    .expected_status(1)
    .run()
    .inspect(|test| {
      let mut executions = test
        .executions()
        .into_iter()
        .map(|execution| {
          (execution.command, execution.exit_code, execution.shell)
        })
        .collect::<Vec<_>>();

      executions.sort();

      assert_eq!(
        executions,
        [
          ("false".into(), Some(1), Some("bash".into())),
          ("true".into(), Some(0), Some("bash".into())),
        ],
      );
    });
}

#[test]
#[ignore = "requires bash"]
fn bash_respects_history_exclusions() {
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
        HISTCONTROL=ignoreboth
        HISTIGNORE=false
        PS1=
        PS2=
        eval "$(honu init bash)"
        "#
      },
    )
    .stdin(" true\nfalse\ntrue\ntrue\nset +o history\nfalse\nset -o history\nbuiltin true\nexit\n")
    .run()
    .inspect(|test| {
      assert_eq!(
        test
          .executions()
          .into_iter()
          .map(|execution| execution.command)
          .collect::<Vec<_>>(),
        ["true", "set +o history", "builtin true"],
      );
    });
}

#[test]
#[ignore = "requires fish"]
fn fish_init() {
  Test::new()
    .program("fish")
    .argument("-n")
    .stdin(include_str!("../src/shell/fish/init.fish"))
    .run();
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
    .run()
    .inspect(|test| assert!(!test.path("honu/history.db").exists()));
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
    .run()
    .inspect(|test| {
      let mut executions = test
        .executions()
        .into_iter()
        .map(|execution| {
          (execution.command, execution.exit_code, execution.shell)
        })
        .collect::<Vec<_>>();

      executions.sort();

      assert_eq!(
        executions,
        [
          ("false".into(), Some(1), Some("fish".into())),
          ("true".into(), Some(0), Some("fish".into())),
        ],
      );
    });
}

#[test]
#[ignore = "requires zsh"]
fn zsh_init() {
  Test::new()
    .program("zsh")
    .argument("-n")
    .stdin(include_str!("../src/shell/zsh/init.zsh"))
    .run();
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
    .expected_status(1)
    .run()
    .inspect(|test| {
      let mut executions = test
        .executions()
        .into_iter()
        .map(|execution| {
          (execution.command, execution.exit_code, execution.shell)
        })
        .collect::<Vec<_>>();

      executions.sort();

      assert_eq!(
        executions,
        [
          ("false".into(), Some(1), Some("zsh".into())),
          ("true".into(), Some(0), Some("zsh".into())),
        ],
      );
    });
}

#[test]
#[ignore = "requires zsh"]
fn zsh_respects_history_exclusions() {
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
        setopt HIST_IGNORE_SPACE
        _ignore_history() {
          [[ "$1" == *ignored-by-hook* ]] && return 1
          return 0
        }
        autoload -Uz add-zsh-hook
        add-zsh-hook zshaddhistory _ignore_history
        eval "$(honu init zsh)"
        "#
      },
    )
    .stdin(" : ignored-by-space\n: ignored-by-hook\nfalse\ntrue\nexit\n")
    .run()
    .inspect(|test| {
      let mut executions = test
        .executions()
        .into_iter()
        .map(|execution| {
          (execution.command, execution.exit_code, execution.shell)
        })
        .collect::<Vec<_>>();

      executions.sort();

      assert_eq!(
        executions,
        [
          ("false".into(), Some(1), Some("zsh".into())),
          ("true".into(), Some(0), Some("zsh".into())),
        ],
      );
    });
}
