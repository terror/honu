use super::*;

const SPINNER_STYLE: &str = "{spinner:.green} ⟪{elapsed_precise}⟫ \
  {binary_bytes} ⟨{binary_bytes_per_sec}⟩ {msg}";

pub(super) struct Progress {
  bar: ProgressBar,
}

impl Progress {
  pub(super) fn finish(self) {
    self.bar.finish_and_clear();
  }

  pub(super) fn new(message: impl Into<Cow<'static, str>>) -> Result<Self> {
    let bar = ProgressBar::new_spinner()
      .with_style(ProgressStyle::with_template(SPINNER_STYLE)?)
      .with_message(message);

    if !bar.is_hidden() {
      bar.enable_steady_tick(Duration::from_millis(50));
    }

    Ok(Self { bar })
  }

  pub(super) fn reader<R: Read>(&self, reader: R) -> ProgressBarIter<R> {
    self.bar.wrap_read(reader)
  }

  pub(super) fn set_message(&self, message: impl Into<Cow<'static, str>>) {
    self.bar.set_message(message);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn styles_are_valid() {
    ProgressStyle::with_template(SPINNER_STYLE).unwrap();
  }
}
