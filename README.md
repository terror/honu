## honu

<img align="right" width="125" height="125" src="etc/icon.png">

[![release](https://img.shields.io/github/release/terror/honu.svg?label=release&style=flat&labelColor=1d1d1d&color=424242&logo=github)](https://github.com/terror/honu/releases/latest)
[![crates.io](https://img.shields.io/crates/v/honu.svg?style=flat&labelColor=1d1d1d&color=424242&logo=rust)](https://crates.io/crates/honu)
[![build](https://img.shields.io/github/actions/workflow/status/terror/honu/ci.yaml?branch=master&style=flat&labelColor=1d1d1d&color=424242&logo=GitHub%20Actions&logoColor=white&label=build)](https://github.com/terror/honu/actions/workflows/ci.yaml)
[![codecov](https://img.shields.io/codecov/c/gh/terror/honu?style=flat&labelColor=1d1d1d&color=424242&logo=Codecov&logoColor=white)](https://codecov.io/gh/terror/honu)
[![downloads](https://img.shields.io/github/downloads/terror/honu/total.svg?style=flat&labelColor=1d1d1d&color=424242)](https://github.com/terror/honu/releases)

`honu` records, imports, and searches your shell history with SQLite.

<img width="1667" alt="val" src="screenshot.png" />

If you need help with `honu` please feel free to open an issue. Feature requests
and bug reports are always welcome!

## Installation

`honu` should run on any system, including Linux, MacOS, and Windows.

The easiest way to install it is by using
[cargo](https://doc.rust-lang.org/cargo/index.html), the Rust package manager:

```bash
cargo install honu
```

Otherwise, see below for the complete package list:

#### Cross-platform

<table>
  <thead>
    <tr>
      <th>Package Manager</th>
      <th>Package</th>
      <th>Command</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href=https://www.rust-lang.org>Cargo</a></td>
      <td><a href=https://crates.io/crates/honu>honu</a></td>
      <td><code>cargo install honu</code></td>
    </tr>
    <tr>
      <td><a href=https://brew.sh>Homebrew</a></td>
      <td><a href=https://github.com/terror/homebrew-tap>terror/tap/honu</a></td>
      <td><code>brew install terror/tap/honu</code></td>
    </tr>
  </tbody>
</table>

### Pre-built binaries

Pre-built binaries for Linux, MacOS, and Windows can be found on
[the releases page](https://github.com/terror/honu/releases).

## Usage

Add the following command to your `.zshrc` to record commands and bind
`control-r` to interactive history search:

```zsh
eval "$(honu init zsh)"
```

Shell integration is also available for
[Bash](<https://en.wikipedia.org/wiki/Bash_(Unix_shell)?useskin=vector>) and
[Fish](https://fishshell.com/) through `honu init`.

To seed the database with your existing shell history, run:

```bash
honu import
```

The shell is detected automatically, or may be passed explicitly with
`honu import bash`, `honu import fish`, or `honu import zsh`.

Press `control-r` to search your recorded history. The current command line is
used as the initial query, and the selected command replaces it without being
executed. You can also start a search directly:

```bash
honu search
honu search cargo
```

Use `honu list` to print recent commands and `honu backup` to copy the database:

```bash
honu list --limit 20
honu backup history.db
```

You can check out the
[full command specification](https://github.com/terror/honu/tree/master/src/subcommand),
or invoke `honu --help` for more information!

## Configuration

On Linux and macOS, `honu` reads its TOML configuration from
`$XDG_CONFIG_HOME/honu/config.toml`, or `~/.config/honu/config.toml` when
`XDG_CONFIG_HOME` is unset.

On Windows, it reads `%XDG_CONFIG_HOME%\honu\config.toml`, or
`%APPDATA%\honu\config.toml` when `XDG_CONFIG_HOME` is unset.

Here is a sample configuration file:

```toml
[import]
shell = "zsh"

[search]
case = "smart"
directory_width = 16
height = 60
info = "right"
limit = 10000
mode = "fuzzy"
prompt = " > "

[theme]
accent = 6
```

### Import

The configured import shell may be `bash`, `fish`, or `zsh`. An explicit shell
passed to `honu import` takes precedence over this setting.

### Search

Search uses 60% of the terminal height by default. The configured height must be
between 1 and 100. There is no default result limit, and `--limit` overrides the
configured limit.

Case matching may be `smart`, `sensitive`, or `insensitive`, and defaults to
`smart`. Search mode may be `fuzzy`, `exact`, or `regex`, and defaults to
`fuzzy`.

Directory names are displayed in a 16-column field by default. Info placement
may be `default`, `hidden`, `inline`, `inline-right`, `left`, or `right`. The
prompt defaults to `>`.

### Theme

The theme accent sets the ANSI color used for the selected row, matches, query,
prompt, and cursor in interactive search. It may be any ANSI color index from 0
to 255 and defaults to 6 (cyan).

## Prior Art

This project was inspired by tools like
[atuin](https://github.com/atuinsh/atuin) and
[stinkpot](https://tangled.org/oppi.li/stinkpot).
