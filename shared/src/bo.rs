pub mod artifact;
mod manifest;
pub(crate) mod dao;
use crate::error;
use diesel::{Connection, PgConnection};
use artifact::{ArtifactRequest, Rollout};

pub use dao::{initialize_db_pool, ConnectionPool};

pub use dao::get_connection;

use self::dao::model;
use artifact::{Validable, ArtifactValidator};

pub struct ArtifactOps;

impl ArtifactOps {
    /// Create the artifact object in database
    pub fn create(conn: &mut PgConnection, token: &str, mut req: ArtifactRequest) -> error::Result<i32> {
        let mut validator = ArtifactValidator{conn, artifact: &mut req};
        validator.validate()?;
        req.format()?;
        let team = dao::TeamDao::find_team_by_token(conn, token)?;
        match team.id {
            Some(team_id) => {
                let art = dao::model::Artifact {
                    id: None,
                    name: req.name,
                    total: req.total,
                    target: req.target,
                    team_id, // TODO: get team id first
                    build: serde_json::to_value(req.build)?,
                    clean: Some(serde_json::to_value(req.clean)?)
                };

                conn.transaction(|connection| {
                    dao::ArtifactDao::create(connection, art)
                })
            },
            None => {
                Err(error::error("The team token does not match any team"))
            }
        }
    }

    pub fn update(conn: &mut PgConnection, token: &str, mut req: ArtifactRequest) -> error::Result<()> {
        let mut validator = ArtifactValidator{conn, artifact: &mut req};
        validator.validate()?;
        req.format()?;
        let team = dao::TeamDao::find_team_by_token(conn, token)?;
        match team.id {
            Some(team_id) => {
                let art = dao::model::Artifact {
                    id: None,
                    name: req.name,
                    total: req.total,
                    target: req.target,
                    team_id,
                    build: serde_json::to_value(req.build)?,
                    clean: Some(serde_json::to_value(req.clean)?)
                };

                // TODO: notify the engine that the new art is ready.
                // TODO: save the create script and destroy script
                conn.transaction(|connection| {
                    dao::ArtifactDao::update(connection, art)
                })
            },
            None => {
                Err(error::error("The team token does not match any team"))
            }
        }
    }

    pub fn update_build_script(_conn: &mut PgConnection, mut req: Rollout) -> error::Result<()> {
        req.validate()?;
        req.format()?;
        Ok(())
    }

    pub fn update_clean_script(_conn: &mut PgConnection, mut req: Rollout) -> error::Result<()> {
        req.validate()?;
        req.format()?;
        Ok(())
    }


    pub fn delete(_id: &str) -> error::Result<()> {
        Ok(())
    }

    pub fn load_by_id(conn: &mut PgConnection, id: i32) -> error::Result<model::Artifact> {
        dao::ArtifactDao::load_by_id(conn, id)
    }

    pub fn load_by_name(conn: &mut PgConnection, name: String) -> error::Result<model::Artifact> {
        dao::ArtifactDao::load_by_name(conn, name)
    }

    pub fn deploy(_id: &str) -> error::Result<()> {
        Ok(())
    }

    pub fn destroy(_id: &str) -> error::Result<()> {
        Ok(())
    }
}

pub struct TeamOps;
impl TeamOps {
    pub fn create(conn: &mut PgConnection, team_name: String, desp: Option<String>) -> error::Result<i32> {
        let token = Self::generate();
        let team = dao::model::Team {
            id: None,
            name: team_name,
            token,
            desp
        };

        dao::TeamDao::create(conn, team)
    }

    pub fn find_team_by_token(conn: &mut PgConnection, team_token: String) -> error::Result<dao::model::Team> {
        dao::TeamDao::find_team_by_token(conn, &team_token)
    }

    pub fn find_team_by_name(conn: &mut PgConnection, team_name: String) -> error::Result<dao::model::Team> {
        dao::TeamDao::find_team_by_name(conn, &team_name)
    }

    pub fn find_team_by_token_for_update(conn: &mut PgConnection, team_token: String) -> error::Result<dao::model::Team> {
        dao::TeamDao::find_team_by_token_for_update(conn, &team_token)
    }

    pub fn find_team_by_name_for_update(conn: &mut PgConnection, team_name: String) -> error::Result<dao::model::Team> {
        dao::TeamDao::find_team_by_name_for_update(conn, &team_name)
    }

    pub fn delete(conn: &mut PgConnection, team_id: i32) -> error::Result<usize> {
        dao::TeamDao::delete(conn, team_id)
    }

    pub fn rotate_token(conn: &mut PgConnection, team_name: String) -> error::Result<()> {
        let token = Self::generate();
        dao::TeamDao::rotate_token(conn, team_name, token)
    }

    fn generate() -> String {
        //TODO: generate the token
        String::from("123456")
    }
}

pub mod tests {
    use diesel::{Connection, PgConnection};

    use super::dao::{ArtifactDao, TeamDao, get_connection};
    use crate::error;

    pub struct Environment;
    impl Environment {
        pub fn init(clean: bool, func: impl FnOnce(&mut PgConnection) -> error::Result<()>) -> error::Result<()> {
            let mut conn = get_connection();
            conn.test_transaction(|connection| {
                if clean {
                    Self::clean(connection).unwrap();
                }
                func(connection)
            });
            Ok(())
        }

        fn clean(conn: &mut PgConnection) -> error::Result<()> {
            // Clean artifact
            ArtifactDao::delete_all(conn).expect("Failed to clean artifact");
            // Clean team
            TeamDao::delete_all(conn).expect("Failed to clean team");
            Ok(())
        }

    }

    pub fn clean(conn: &mut PgConnection) {
        ArtifactDao::delete_all(conn).expect("Failed to clean artifact");
        // Clean team
        TeamDao::delete_all(conn).expect("Failed to clean team");
    }
}
