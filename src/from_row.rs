use super::*;

pub trait FromRow: Sized {
  /// Constructs a value from `row`.
  ///
  /// # Errors
  ///
  /// Returns an error if a column is missing or cannot be converted to its
  /// corresponding field type.
  fn from_row(row: &Row<'_>) -> Result<Self>;
}
