use crate::domain::entities::User;
use crate::domain::valueobjects::{NickName, RawPassword, Salt, SaltedPassword, UserId};
use uuid::Uuid;

pub struct UserFactory {
  password_salt: Salt,
}

impl UserFactory {
  pub fn new(password_salt: Salt) -> Self {
    Self { password_salt }
  }

  pub fn create(
    &self, nickname: impl Into<NickName>, password: &RawPassword,
    password_change_deadline: chrono::DateTime<chrono::Utc>,
  ) -> User {
    let uuid = Uuid::new_v4();
    let user_id = UserId::from(uuid);
    let nickname = nickname.into();
    let salted_password = SaltedPassword::new(password, &self.password_salt);
    User::new(user_id, nickname, salted_password, password_change_deadline)
  }

  pub fn password_salt(&self) -> &Salt {
    &self.password_salt
  }
}
