/// Pagination parameters for querying lists
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
  pub limit: usize,
  pub offset: usize,
}

impl Pagination {
  /// Default limit value
  const DEFAULT_LIMIT: usize = 100;
  /// Default offset value
  const DEFAULT_OFFSET: usize = 0;

  /// Create a Pagination from optional limit and offset.
  /// Uses defaults (limit: 100, offset: 0) for None values.
  pub fn from_options(limit: Option<usize>, offset: Option<usize>) -> Self {
    Self {
      limit: limit.unwrap_or(Self::DEFAULT_LIMIT),
      offset: offset.unwrap_or(Self::DEFAULT_OFFSET),
    }
  }
}

impl Default for Pagination {
  fn default() -> Self {
    Self {
      limit: Pagination::DEFAULT_LIMIT,
      offset: Pagination::DEFAULT_OFFSET,
    }
  }
}
