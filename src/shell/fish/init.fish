if not set -q HONU_SESSION; or test -z "$HONU_SESSION"; or test "$HONU_SHLVL" != "$SHLVL"
  set -gx HONU_SESSION (string join : "$hostname" "$fish_pid" (command date +%s))
  set -gx HONU_SHLVL "$SHLVL"
end

set -g __honu_command
set -g __honu_directory
set -g __honu_started_at_ns
set -g __honu_timestamp_second
set -g __honu_timestamp_sequence 0

function _honu_preexec --on-event fish_preexec
  if test -n "$fish_private_mode"
    set -e __honu_command
    return
  end

  if functions -q fish_should_add_to_history
    if not fish_should_add_to_history "$argv[1]"
      set -e __honu_command
      return
    end
  else if string match -qr '^ ' -- "$argv[1]"
    set -e __honu_command
    return
  end

  set -g __honu_command "$argv[1]"
  set -g __honu_directory "$PWD"
  set -l started_at (command date +%s)

  if test "$started_at" = "$__honu_timestamp_second"
    set -g __honu_timestamp_sequence (math "$__honu_timestamp_sequence + 1")
  else
    set -g __honu_timestamp_second "$started_at"
    set -g __honu_timestamp_sequence 0
  end

  set -l timestamp_suffix (printf '%09d' "$__honu_timestamp_sequence")
  set -g __honu_started_at_ns (string join '' "$started_at" "$timestamp_suffix")
end

function _honu_postexec --on-event fish_postexec
  set -l exit_code $status
  set -l command "$__honu_command"
  set -l directory "$__honu_directory"
  set -l timestamp_ns "$__honu_started_at_ns"
  set -l arguments \
    --directory "$directory" \
    --exit-code "$exit_code" \
    --session "$HONU_SESSION" \
    --shell fish

  set -e __honu_command
  set -e __honu_directory
  set -e __honu_started_at_ns

  if test -z "$command"
    return "$exit_code"
  end

  if test -n "$timestamp_ns"; and set -q CMD_DURATION
    set -l duration_ns (math --scale=0 "$CMD_DURATION * 1000000")
    set -a arguments --timestamp-ns "$timestamp_ns" --duration-ns "$duration_ns"
  end

  if test -n "$hostname"
    set -a arguments --hostname "$hostname"
  end

  command honu add $arguments -- "$command" >/dev/null 2>&1

  return "$exit_code"
end

function _honu_search
  set -l query (commandline | string collect)
  set -l selected (command honu search --interactive -- "$query" | string collect)
  set -l exit_code $pipestatus[1]

  if test "$exit_code" -eq 0; and test -n "$selected"
    commandline --replace "$selected"
    commandline --cursor (string length -- "$selected")
  end

  commandline --function repaint

  return "$exit_code"
end

bind ctrl-r _honu_search
bind --mode insert ctrl-r _honu_search
