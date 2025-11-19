use std::ops::{Index, IndexMut};

use crate::domain::PlayerIndex;

pub struct PlayerIndexedVec<T> {
  values: Vec<T>,
}

impl<T> Default for PlayerIndexedVec<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> PlayerIndexedVec<T> {
  pub fn new() -> Self {
    Self { values: Vec::new() }
  }

  pub fn with_len(len: usize) -> Self
  where
    T: Default,
  {
    let mut v = Vec::new();
    v.resize_with(len, || T::default());
    Self { values: v }
  }

  pub fn from4(v1: T, v2: T, v3: T, v4: T) -> Self {
    Self {
      values: vec![v1, v2, v3, v4],
    }
  }

  pub fn from6(v1: T, v2: T, v3: T, v4: T, v5: T, v6: T) -> Self {
    Self {
      values: vec![v1, v2, v3, v4, v5, v6],
    }
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }

  pub fn is_empty(&self) -> bool {
    self.values.is_empty()
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.values.iter()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
    self.values.iter_mut()
  }

  pub fn split_at_mut(&mut self, mid: PlayerIndex) -> (&mut [T], &mut [T]) {
    self.values.split_at_mut(mid.value())
  }

  pub fn push(&mut self, value: T) {
    self.values.push(value);
  }
}

impl<T> Index<PlayerIndex> for PlayerIndexedVec<T> {
  type Output = T;

  fn index(&self, index: PlayerIndex) -> &Self::Output {
    &self.values[index.value()]
  }
}

impl<T> IndexMut<PlayerIndex> for PlayerIndexedVec<T> {
  fn index_mut(&mut self, index: PlayerIndex) -> &mut Self::Output {
    &mut self.values[index.value()]
  }
}
