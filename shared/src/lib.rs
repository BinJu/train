pub mod error;
pub mod queue;
pub mod scheduler;
pub mod bo;
mod command;

#[cfg(test)]
mod tests {
    use crate::bo::dao::{TeamDao, ArtifactDao};
    use crate::bo::artifact::ArtifactRequest;
    use diesel::pg::PgConnection;
    use crate::bo::dao::model;
    use crate::error;
    // POST   /api/v1/art with body in json as request

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
    fn test_artifact_creation() {
        crate::bo::tests::Environment::init(true, |conn| {
            run_case(conn, |conn| {
                let file = std::fs::File::open("../asset/sample-artifact-request.json").unwrap();

                let mut artifact_request: ArtifactRequest = serde_json::from_reader(file).expect("Fail to parse the json ArtifactRequest");
                artifact_request.name = "test-lib-artifact-creation".to_owned();
                let team = TeamDao::find_team_by_token(conn, "234567").expect("Unable to find the team witht the token 123456");
                let artifact = model::Artifact {
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
    fn test_artifact_list() {

    }

    #[test]
    fn test_artifact_borrow() {

    }

    #[test]
    fn test_artfact_return() {

    }

    #[test]
    fn test_artifact_describe() {

    }

    #[test]
    fn test_artifact_update() {

    }

    #[test]
    fn test_artifact_destroy() {

    }

    #[test]
    fn test_secret_creation() {

    }

    #[test]
    fn test_secret_list() {

    }

    #[test]
    fn test_secret_show() {

    }

    #[test]
    fn test_secret_update() {

    }

    #[test]
    fn test_secret_destroy() {

    }

    #[test]
    fn test_account_creation() {

    }

    #[test]
    fn test_account_list() {

    }

    #[test]
    fn test_account_show() {

    }

    #[test]
    fn test_account_update() {

    }

    #[test]
    fn test_account_destroy() {

    }

}
