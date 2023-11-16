mod artifact_dao;
mod instance_dao;

pub use artifact_dao::ArtifactDao;
pub use instance_dao::InstanceDao;

pub const DEFAULT_REDIS_URL: &str = "redis://train-redis";

use crate::error;

pub fn connection(url: &str) -> error::Result<redis::Connection> {
    let conn = redis::Client::open(url)?.get_connection()?;
    Ok(conn)
}

