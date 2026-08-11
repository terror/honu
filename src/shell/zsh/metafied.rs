use super::*;

const META: u8 = 0x83;

pub(crate) struct Metafied<R> {
  pub(crate) escaped: bool,
  pub(crate) reader: R,
}

impl<R: BufRead> Read for Metafied<R> {
  fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
    if output.is_empty() {
      return Ok(0);
    }

    let mut written = 0;

    while written < output.len() {
      let (consumed, exhausted) = {
        let input = self.reader.fill_buf()?;

        if input.is_empty() {
          (0, true)
        } else {
          let mut consumed = 0;

          for byte in input.iter().copied() {
            consumed += 1;

            if self.escaped {
              output[written] = byte ^ 0x20;
              written += 1;
              self.escaped = false;
            } else if byte == META {
              self.escaped = true;
            } else {
              output[written] = byte;
              written += 1;
            }

            if written == output.len() {
              break;
            }
          }

          (consumed, false)
        }
      };

      self.reader.consume(consumed);

      if exhausted {
        if self.escaped {
          if written > 0 {
            return Ok(written);
          }

          return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "zsh history ends with an incomplete metafied byte",
          ));
        }

        break;
      }
    }

    Ok(written)
  }
}
