use super::*;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
  pub(crate) import: Import,
  pub(crate) search: Search,
  pub(crate) theme: Theme,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Import {
  pub(crate) shell: Option<Shell>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Search {
  pub(crate) height: u8,
  pub(crate) limit: Option<usize>,
}

impl Default for Search {
  fn default() -> Self {
    Self {
      height: 60,
      limit: None,
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Theme {
  pub(crate) accent: u8,
}

impl Default for Theme {
  fn default() -> Self {
    Self { accent: 6 }
  }
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

    let config: Self =
      confy::load_path(path).context("failed to load configuration")?;

    if config.search.height > 100 {
      bail!("search height must be between 0 and 100");
    }

    Ok(config)
  }
}
