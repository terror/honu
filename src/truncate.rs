use super::*;

pub(crate) trait Truncate {
  fn truncate(&self, width: usize) -> Cow<'_, str>;
}

impl Truncate for str {
  fn truncate(&self, width: usize) -> Cow<'_, str> {
    let max_width = u64::try_from(width).unwrap();

    if unicode_display_width::width(self) <= max_width {
      return Cow::Borrowed(self);
    }

    let Some(content_width) = max_width.checked_sub(1) else {
      return Cow::Borrowed("");
    };

    let end = self
      .grapheme_indices(true)
      .scan(0, |used, (index, grapheme)| {
        *used += unicode_display_width::width(grapheme);
        (*used <= content_width).then_some(index + grapheme.len())
      })
      .last()
      .unwrap_or(0);

    Cow::Owned(format!("{}…", &self[..end]))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn truncate() {
    #[track_caller]
    fn case(input: &str, width: usize, expected: &str) {
      assert_eq!(input.truncate(width), expected);
    }

    case("foo", 3, "foo");
    case("foobar", 4, "foo…");
    case("foo界界", 6, "foo界…");
    case("foo界界", 5, "foo…");
    case("foo", 0, "");
  }
}
