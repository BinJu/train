use crate::error;
use diesel::pg::PgConnection;
use super::model;

pub struct ArtifactDao;

impl ArtifactDao {
    pub fn create(conn: &mut PgConnection, art: model::Artifact) -> error::Result<i32> {
        use super::schema::artifact::dsl::*;
        use diesel::prelude::*;
        // Save the artifact
        diesel::insert_into(artifact)
            .values(&art)
            .returning(id)
            .get_result(conn)
            .map_err(|err| err.into())
    }

    pub fn load_by_name(conn: &mut PgConnection, art_name: String) -> error::Result<model::Artifact> {
        use super::schema::artifact::dsl::*;
        use diesel::prelude::*;
        artifact.filter(name.eq(art_name))
            .select(model::Artifact::as_select())
            .first(conn)
            .map_err(|err| err.into())
    }

    pub fn load_by_id(conn: &mut PgConnection, art_id: i32) -> error::Result<model::Artifact> {
        use super::schema::artifact::dsl::*;
        use diesel::prelude::*;
        artifact.filter(id.eq(art_id))
            .select(model::Artifact::as_select())
            .first(conn)
            .map_err(|err| err.into())
    }

    pub fn update(conn: &mut PgConnection, art: model::Artifact) -> error::Result<()> {
        use super::schema::artifact::dsl::*;
        use diesel::prelude::*;
        // Save the artifact
        // TODO: find by name if id is null
        diesel::update(artifact)
            .filter(id.eq(art.id.unwrap()))
            .set(&art)
            .execute(conn)
            .expect("Error saving new artifact");
        Ok(())
    }

    pub fn update_build_script(_conn: &mut PgConnection, _art_id: i32, _rollout: serde_json::Value) -> error::Result<i32> {
        Ok(0)
    }

    pub fn update_clean_script(_conn: &mut PgConnection, _art_id: i32, _rollout: serde_json::Value) -> error::Result<i32> {
        Ok(0)
    }

    pub fn delete_by_id(conn: &mut PgConnection, art_id: i32) -> error::Result<usize> {
        use diesel::prelude::*;
        use super::schema::artifact::dsl::*;
        diesel::delete(artifact.filter(id.eq(art_id))).execute(conn).map_err(|err|err.into())
    }

    pub fn delete_by_name(conn: &mut PgConnection, art_name: &str) -> error::Result<usize> {
        use diesel::prelude::*;
        use super::schema::artifact::dsl::*;
        diesel::delete(artifact.filter(name.eq(art_name))).execute(conn).map_err(|err|err.into())
    }

    pub fn delete_all(conn: &mut PgConnection) -> error::Result<usize> {
        use diesel::prelude::*;
        use super::schema::artifact::dsl::*;
        diesel::delete(artifact).execute(conn).map_err(|err|err.into())
    }

    pub fn exist_name(conn: &mut PgConnection, art_name: &str) -> error::Result<bool> {
        use super::schema::artifact::dsl::*;
        use diesel::prelude::*;
        diesel::dsl::select(diesel::dsl::exists(artifact.filter(name.eq(art_name)))).get_result(conn).map_err(|err| err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bo::ArtifactRequest;
    use super::model::Artifact;
    use crate::bo::dao::TeamDao;

    fn run_case(conn: &mut PgConnection, case: impl FnOnce(&mut PgConnection) -> error::Result<()>)-> error::Result<()>  {
        let team = model::Team {
            id: None,
            name: "Team C".to_owned(),
            token: "234567".to_owned(),
            desp: None
        };
        match TeamDao::create(conn, team) {
            Ok(team_id) => {
                case(conn).expect("Fail to run the test case");
            },
            Err(err) => {
                println!("Failed to create team: {:?}", err);
                panic!();
            }
        };
        Ok(())
    }

    #[test]
    fn test_dao_create_artifact() {
        crate::bo::tests::Environment::init(true, |conn| {
            run_case(conn, |conn| {
                let file = std::fs::File::open("../asset/sample-artifact-request.json").unwrap();

                let artifact_request: ArtifactRequest = serde_json::from_reader(file).expect("Fail to parse the json ArtifactRequest");
                let team = TeamDao::find_team_by_token(conn, "234567").expect("Unable to find the team witht the token 123456");
                let artifact = Artifact {
                    id: None,
                    name: artifact_request.name,
                    total: artifact_request.total,
                    target: artifact_request.target,
                    team_id: team.id.expect("Null team Id"),
                    build: serde_json::to_value(artifact_request.build).expect("failed to convert artifact_request.build to serde_json.value"),
                    clean: Some(serde_json::to_value(artifact_request.clean).expect("failed to convert artifact_request.build to serde_json.value"))
                };
                let result = ArtifactDao::create(conn, artifact);
                match result {
                    Err(err) => {
                        println!("Encountered err: {err}");
                        panic!();
                    },
                    Ok(data) => {
                        assert!(data.is_positive());
                    }
                };
                Ok(())
                
            })
        }).unwrap();
    }

    #[test]
    fn test_dao_find_by_name() {
        crate::bo::tests::Environment::init(true, |conn| {
            run_case(conn, |conn| { 
                let file = std::fs::File::open("../asset/sample-artifact-request.json").unwrap();

                let artifact_request: ArtifactRequest = serde_json::from_reader(file).expect("Fail to parse the json ArtifactRequest");
                let team = TeamDao::find_team_by_token(conn, "234567").expect("Unable to find the team witht the token 123456");
                let art_name = artifact_request.name.clone();
                let artifact = Artifact {
                    id: None,
                    name: artifact_request.name,
                    total: artifact_request.total,
                    target: artifact_request.target,
                    team_id: team.id.expect("Null team Id"),
                    build: serde_json::to_value(artifact_request.build).expect("failed to convert artifact_request.build to serde_json.value"),
                    clean: Some(serde_json::to_value(artifact_request.clean).expect("failed to convert artifact_request.build to serde_json.value"))
                };
                let result = ArtifactDao::create(conn, artifact);
                match result {
                    Err(err) => {
                        println!("Encountered err: {err}");
                        panic!();
                    },
                    Ok(data) => {
                        assert!(data.is_positive());
                        let art = ArtifactDao::load_by_name(conn, art_name)?;
                        assert!(art.id.is_some());
                        assert!(art.id.unwrap().is_positive());
                        assert_eq!(art.total, artifact_request.total);
                        assert_eq!(art.target, artifact_request.target);
                    }
                };
                Ok(())
            })
        }).unwrap();
        
    }

    #[test]
    fn test_dao_exist_name() {
        crate::bo::tests::Environment::init(true, |conn| {
            run_case(conn, |conn| { 
                let file = std::fs::File::open("../asset/sample-artifact-request.json").unwrap();

                let artifact_request: ArtifactRequest = serde_json::from_reader(file).expect("Fail to parse the json ArtifactRequest");
                let team = TeamDao::find_team_by_token(conn, "234567").expect("Unable to find the team witht the token 123456");
                let artifact = Artifact {
                    id: None,
                    name: artifact_request.name,
                    total: artifact_request.total,
                    target: artifact_request.target,
                    team_id: team.id.expect("Null team Id"),
                    build: serde_json::to_value(artifact_request.build).expect("failed to convert artifact_request.build to serde_json.value"),
                    clean: Some(serde_json::to_value(artifact_request.clean).expect("failed to convert artifact_request.build to serde_json.value"))
                };
                let result = ArtifactDao::create(conn, artifact);
                match result {
                    Err(err) => {
                        println!("Encountered err: {err}");
                        panic!();
                    },
                    Ok(data) => {
                        assert!(data.is_positive());
                    }
                };
                Ok(())
            })
        }).unwrap();
        
    }
}
