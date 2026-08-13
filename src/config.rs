use super::*;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Config {
  pub(crate) import: Import,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Import {
  pub(crate) shell: Option<Shell>,
}

impl Config {
  pub(crate) fn load() -> Result<Self> {
    #[cfg(unix)]
    let path =
      BaseDirectories::with_prefix("honu").place_config_file("config.toml")?;

    #[cfg(windows)]
    let path = env::var_os("XDG_CONFIG_HOME")
      .map(PathBuf::from)
      .filter(|path| path.is_absolute())
      .or_else(|| env::var_os("APPDATA").map(PathBuf::from))
      .context("failed to determine local config directory")?
      .join("honu/config.toml");

    confy::load_path(path).context("failed to load configuration")
  }
}
