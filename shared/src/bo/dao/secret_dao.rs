use crate::error;
use diesel::pg::PgConnection;

pub struct SecretDao;

impl SecretDao {
    pub fn exist_name(conn: &mut PgConnection, sec_name: &str) -> error::Result<bool> {
        use super::schema::secret::dsl::*;
        use diesel::prelude::*;
        diesel::dsl::select(diesel::dsl::exists(secret.filter(name.eq(sec_name)))).get_result(conn).map_err(|err| err.into())
    }
}
