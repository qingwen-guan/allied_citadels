use uuid::Uuid;

use crate::domain::entities::Account;
use crate::domain::valueobjects::{NickName, RawPassword, Salt, SaltedPassword};

pub struct AccountFactory {
  password_salt: Salt,
}

impl AccountFactory {
  pub fn new(password_salt: Salt) -> Self {
    Self { password_salt }
  }

  pub fn create(&self, nickname: &NickName, password: &RawPassword) -> Account {
    let uuid = Uuid::new_v4();
    let salted_password = SaltedPassword::new(password, &self.password_salt);
    Account::new(uuid, nickname.clone(), salted_password)
  }

  pub fn password_salt(&self) -> &Salt {
    &self.password_salt
  }
}
