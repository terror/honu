# Changelog

## [0.1.2](https://github.com/terror/honu/releases/tag/0.1.2) - 2026-08-13

### Added

- Add section headers to configuration documentation ([#96](https://github.com/terror/honu/pull/96) by [terror](https://github.com/terror))
- Add usage section to readme ([#95](https://github.com/terror/honu/pull/95) by [terror](https://github.com/terror))
- Add configurable search info placement ([#92](https://github.com/terror/honu/pull/92) by [terror](https://github.com/terror))
- Add configurable search directory width ([#91](https://github.com/terror/honu/pull/91) by [terror](https://github.com/terror))
- Add configurable search mode ([#90](https://github.com/terror/honu/pull/90) by [terror](https://github.com/terror))
- Add configurable search case ([#89](https://github.com/terror/honu/pull/89) by [terror](https://github.com/terror))
- Add configurable search prompt ([#88](https://github.com/terror/honu/pull/88) by [terror](https://github.com/terror))
- Add configurable search defaults ([#87](https://github.com/terror/honu/pull/87) by [terror](https://github.com/terror))
- Add configurable search accent ([#86](https://github.com/terror/honu/pull/86) by [terror](https://github.com/terror))
- Add import shell configuration ([#84](https://github.com/terror/honu/pull/84) by [terror](https://github.com/terror))

### Fixed

- Pass search config to item loader ([#93](https://github.com/terror/honu/pull/93) by [terror](https://github.com/terror))
- Respect `zsh` history exclusions ([#79](https://github.com/terror/honu/pull/79) by [terror](https://github.com/terror))
- Respect `bash` history exclusions ([#78](https://github.com/terror/honu/pull/78) by [terror](https://github.com/terror))
- Use lowercase shell names when importing ([#77](https://github.com/terror/honu/pull/77) by [terror](https://github.com/terror))
- Detect shell when importing history ([#76](https://github.com/terror/honu/pull/76) by [terror](https://github.com/terror))
- Respect `HISTFILE` when importing history ([#75](https://github.com/terror/honu/pull/75) by [terror](https://github.com/terror))

### Misc

- Clarify configuration paths and search settings ([#94](https://github.com/terror/honu/pull/94) by [terror](https://github.com/terror))
- Move binaries section into installation ([#85](https://github.com/terror/honu/pull/85) by [terror](https://github.com/terror))
- Bump `skim` from 5.6.1 to 5.6.3 ([#80](https://github.com/terror/honu/pull/80) by [dependabot](https://github.com/dependabot))
- Bump `rusqlite` from 0.40.1 to 0.40.2 ([#81](https://github.com/terror/honu/pull/81) by [dependabot](https://github.com/dependabot))
- Bump `clap` from 4.6.5 to 4.6.6 ([#82](https://github.com/terror/honu/pull/82) by [dependabot](https://github.com/dependabot))
- Bump `thiserror` from 2.0.19 to 2.0.20 ([#83](https://github.com/terror/honu/pull/83) by [dependabot](https://github.com/dependabot))

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
