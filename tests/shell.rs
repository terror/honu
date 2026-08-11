use super::*;

impl Test {
  fn assert_shell_executions(&self, shell: &str, hostname: Option<&str>) {
    let directory = self.path("").canonicalize().unwrap();
    let mut executions = self.executions();

    executions.sort_by(|left, right| left.command.cmp(&right.command));

    assert_eq!(executions.len(), 2);

    for (execution, (command, exit_code)) in
      executions.iter().zip([("false", 1), ("true", 0)])
    {
      assert!(execution.timestamp_ns > 0);
      assert!(execution.duration_ns.is_some_and(|duration| duration >= 0));
      assert!(
        execution
          .hostname
          .as_ref()
          .is_some_and(|hostname| !hostname.is_empty())
      );

      assert_eq!(execution.command, command);
      assert_eq!(execution.directory.as_ref(), Some(&directory));
      assert_eq!(execution.exit_code, Some(exit_code));
      assert_eq!(execution.session.as_deref(), Some("bar"));
      assert_eq!(execution.shell.as_deref(), Some(shell));

      if let Some(hostname) = hostname {
        assert_eq!(execution.hostname.as_deref(), Some(hostname));
      }
    }
  }
}

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
        HOSTNAME=baz
        export HONU_SESSION=bar HONU_SHLVL=$SHLVL
        eval "$(honu init bash)"
        "#
      },
    )
    .stdin("true\nfalse\nexit\n")
    .expected_status(1)
    .run()
    .inspect(|test| test.assert_shell_executions("bash", Some("baz")));
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
    .inspect(|test| assert_eq!(test.executions().len(), 0));
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
      set -gx HONU_SESSION bar
      set -gx HONU_SHLVL $SHLVL
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
    .inspect(|test| test.assert_shell_executions("fish", None));
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
         HOST=baz
         typeset -gx HONU_SESSION=bar HONU_SHLVL=$SHLVL
         eval "$(honu init zsh)"
        "#
      },
    )
    .stdin("true\nfalse\nexit\n")
    .expected_status(1)
    .run()
    .inspect(|test| test.assert_shell_executions("zsh", Some("baz")));
}
