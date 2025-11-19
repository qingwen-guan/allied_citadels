use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct IdGen {
  // TODO: use atomic
  data: Arc<Mutex<u32>>,
}

impl Default for IdGen {
  fn default() -> Self {
    Self::new()
  }
}

impl IdGen {
  pub fn new() -> Self {
    Self {
      data: Arc::new(Mutex::new(u32::default())),
    }
  }

  pub fn gen_next(&mut self) -> u32 {
    let mut guard = self.data.lock().unwrap();
    let result = *guard;
    *guard += 1;

    result
  }
}
