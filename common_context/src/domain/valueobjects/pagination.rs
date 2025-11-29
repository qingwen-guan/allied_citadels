/// Pagination parameters for querying lists
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
  pub limit: usize,
  pub offset: usize,
}

impl Default for Pagination {
  fn default() -> Self {
    Self { limit: 100, offset: 0 }
  }
}
