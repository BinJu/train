mod artifact_dao;
mod team_dao;
mod account_dao;
mod secret_dao;
pub(crate) mod model;
mod schema;

pub use artifact_dao::ArtifactDao;
pub use account_dao::AccountDao;
pub use secret_dao::SecretDao;
pub use team_dao::TeamDao;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

pub type ConnectionPool = Pool<ConnectionManager<PgConnection>>;
pub fn initialize_db_pool() -> ConnectionPool {
    dotenvy::dotenv().ok();
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(conn_spec);
    Pool::builder().test_on_check_out(true).build(manager).expect("Could not build connection pool")
}

pub fn get_connection() -> PgConnection {
    use diesel::Connection;

    dotenvy::dotenv().ok();
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    diesel::PgConnection::establish(&conn_spec)
        .expect("Failed to establish Postgres connection. Please make sure you have .env under the folder")
}
