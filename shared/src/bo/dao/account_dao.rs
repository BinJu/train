use crate::error;
use diesel::pg::PgConnection;
use super::model;

pub struct AccountDao;

impl AccountDao {
    pub fn exist_name(conn: &mut PgConnection, acnt_name: &str) -> error::Result<bool> {
        use super::schema::account::dsl::*;
        use diesel::prelude::*;
        diesel::dsl::select(diesel::dsl::exists(account.filter(name.eq(acnt_name)))).get_result(conn).map_err(|err| err.into())
    }
}
