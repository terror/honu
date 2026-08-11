# Changelog

## [0.1.1](https://github.com/terror/honu/releases/tag/0.1.1) - 2026-08-11

### Added

- Improve search interface ([#73](https://github.com/terror/honu/pull/73) by [terror](https://github.com/terror))
- Expose `Execution` from library ([#70](https://github.com/terror/honu/pull/70) by [terror](https://github.com/terror))

### Fixed

- Pluralize counted nouns ([#71](https://github.com/terror/honu/pull/71) by [terror](https://github.com/terror))
- Preserve `bash` Ctrl-R capture ([#61](https://github.com/terror/honu/pull/61) by [terror](https://github.com/terror))
- Preserve scalar `bash` prompt commands ([#60](https://github.com/terror/honu/pull/60) by [terror](https://github.com/terror))
- Respect `fish` private mode ([#59](https://github.com/terror/honu/pull/59) by [terror](https://github.com/terror))

### Misc

- Refactor shell parsers ([#72](https://github.com/terror/honu/pull/72) by [terror](https://github.com/terror))
- Generalize integration test assertions ([#69](https://github.com/terror/honu/pull/69) by [terror](https://github.com/terror))
- Split integration test status and run ([#68](https://github.com/terror/honu/pull/68) by [terror](https://github.com/terror))
- Prefix shell test names ([#67](https://github.com/terror/honu/pull/67) by [terror](https://github.com/terror))
- Test shell initialization end to end ([#66](https://github.com/terror/honu/pull/66) by [terror](https://github.com/terror))
- Use pretty assertions in integration tests ([#65](https://github.com/terror/honu/pull/65) by [terror](https://github.com/terror))
- Split database assertions ([#64](https://github.com/terror/honu/pull/64) by [terror](https://github.com/terror))
- Update rustfmt edition to 2024 ([#63](https://github.com/terror/honu/pull/63) by [terror](https://github.com/terror))
- Use consistent cargo build flags ([#62](https://github.com/terror/honu/pull/62) by [terror](https://github.com/terror))
- Add dependabot workflow ([#58](https://github.com/terror/honu/pull/58) by [terror](https://github.com/terror))

## [0.1.0](https://github.com/terror/honu/releases/tag/0.1.0) - 2026-08-05

### Added

- Record shell commands and execution metadata in SQLite
- Import and reconcile Bash, Fish, and Zsh history
- Search command history interactively across platforms
- Back up history databases without clobbering existing files
- Clear recorded history securely
- Initialize shell integrations for Bash, Fish, and Zsh
