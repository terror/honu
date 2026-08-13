use super::*;

#[derive(Debug, Clap)]
pub(crate) struct Search {
  #[arg(short, long)]
  limit: Option<usize>,
  #[arg(default_value = "")]
  query: String,
}

impl Search {
  const BATCH_SIZE: usize = 256;
  const CHANNEL_CAPACITY: usize = 8;

  fn load_items(
    self,
    database: &Database,
    directory_width: usize,
    sender: &SkimItemSender,
  ) -> Result {
    let mut batch: Vec<Arc<dyn SkimItem>> =
      Vec::with_capacity(Self::BATCH_SIZE);

    let mut flush_threshold = 1;

    let now_ns =
      i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())?;

    database.for_each_command(self.limit, |command| {
      batch.push(Arc::new(Choice {
        command,
        directory_width,
        now_ns,
      }));

      if batch.len() < flush_threshold {
        return true;
      }

      let next_batch = Vec::with_capacity(Self::BATCH_SIZE);

      if sender.send(mem::replace(&mut batch, next_batch)).is_err() {
        return false;
      }

      flush_threshold = Self::BATCH_SIZE;

      true
    })?;

    if !batch.is_empty() {
      let _ = sender.send(batch);
    }

    Ok(())
  }

  pub(crate) fn run(mut self) -> Result {
    let config = Config::load()?;

    let database = Database::load()?;

    if !database.has_executions()? {
      return Ok(());
    }

    let accent = config.theme.accent;

    self.limit = self.limit.or(config.search.limit);

    let options = SkimOptionsBuilder::default()
      .case(config.search.case)
      .color(format!(
        "none,\
          current:{accent}:bold,\
          matched:-1:underlined,\
          current_match:{accent}:bold:underlined,\
          query:{accent}:bold,\
          prompt:{accent}:bold,\
          cursor:{accent}:bold,\
          info:-1:dim,\
          spinner:-1:dim"
      ))
      .exact(matches!(config.search.mode, SearchMode::Exact))
      .height(format!("{}%", config.search.height))
      .info("right")
      .multi(false)
      .multi_select_icon("")
      .prompt(config.search.prompt)
      .query(&self.query)
      .regex(matches!(config.search.mode, SearchMode::Regex))
      .selector_icon("")
      .build()?;

    let (sender, receiver) = bounded(Self::CHANNEL_CAPACITY);

    let directory_width = config.search.directory_width;
    let loader = thread::spawn(move || {
      self.load_items(&database, directory_width, &sender)
    });

    let output = Skim::run_with(options, Some(receiver));

    let load_result = loader
      .join()
      .map_err(|_| Error::msg("interactive search loader panicked"))?;

    load_result?;

    let output = output.map_err(|error| Error::msg(error.to_string()))?;

    if output.is_abort {
      return Ok(());
    }

    if let Some(item) = output.selected_items.first() {
      println!("{}", item.output());
    }

    Ok(())
  }
}
