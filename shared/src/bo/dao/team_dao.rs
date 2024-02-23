use crate::error;
use diesel::pg::PgConnection;
use super::model;

pub struct TeamDao;

impl TeamDao {
    pub fn create(conn: &mut PgConnection, team_model: model::Team) -> error::Result<i32> {
        use super::schema::team::dsl::*;
        use diesel::prelude::*;
        diesel::insert_into(team)
            .values(&team_model)
            .returning(id)
            .get_result(conn)
            .map_err(|err| err.into())
    }

    pub fn find_team_by_token(conn: &mut PgConnection, team_token: &str) -> error::Result<model::Team> {
        use super::schema::team::dsl::*;
        use diesel::prelude::*;
        team.filter(token.eq(team_token))
            .select(model::Team::as_select())
            .first(conn)
            .map_err(|err| err.into())
    }

    pub fn find_team_by_name(conn: &mut PgConnection, team_name: &str) -> error::Result<model::Team> {
        use super::schema::team::dsl::*;
        use diesel::prelude::*;
        team.filter(name.eq(team_name))
            .select(model::Team::as_select())
            .first(conn)
            .map_err(|err| err.into())
    }

    pub fn find_team_by_token_for_update(conn: &mut PgConnection, team_token: &str) -> error::Result<model::Team> {
        use super::schema::team::dsl::*;
        use diesel::prelude::*;
        team.filter(token.eq(team_token))
            .select(model::Team::as_select())
            .for_update()
            .first(conn)
            .map_err(|err| err.into())
    }

    pub fn find_team_by_name_for_update(conn: &mut PgConnection, team_name: &str) -> error::Result<model::Team> {
        use super::schema::team::dsl::*;
        use diesel::prelude::*;
        team.filter(name.eq(team_name))
            .select(model::Team::as_select())
            .for_update()
            .first(conn)
            .map_err(|err| err.into())
    }

    pub fn rotate_token(_conn: &mut PgConnection, _team_name: String, _token: String) -> error::Result<()> {
        Err(error::error("not implemented yet"))
    }

    pub fn delete(conn: &mut PgConnection, team_id: i32) -> error::Result<usize> {
        use diesel::prelude::*;
        use super::schema::team::dsl::*;
        diesel::delete(team.filter(id.eq(team_id))).execute(conn).map_err(|err|err.into())
    }

    pub fn delete_all(conn: &mut PgConnection) -> error::Result<usize> {
        use diesel::prelude::*;
        use super::schema::team::dsl::*;
        diesel::delete(team).execute(conn).map_err(|err|err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bo::dao::get_connection;
    
    #[test]
    fn test_create_team() {
        let mut conn = get_connection();
        let team = model::Team {
            id: None,
            name: "Team C".to_owned(),
            token: "random-generated".to_owned(),
            desp: None
        };
        match TeamDao::create(&mut conn, team) {
            Ok(team_id) => {
                TeamDao::delete(&mut conn, team_id).expect("Failed to remove the test record");
            },
            Err(err) => {
                println!("Failed to create team: {:?}", err);
                panic!();
            }
        }
    }

    #[test]
    fn test_find_team_by_token() {
        let mut conn = get_connection();
        let token = "random-generated-token".to_owned();
        let team = model::Team {
            id: None,
            name: "Team J".to_owned(),
            token: token.clone(),
            desp: None
        };
        let team_id = TeamDao::create(&mut conn, team).expect("Failed to create team: Team J");
        match TeamDao::find_team_by_token(&mut conn, &token) {
            Ok(_) => {
            },
            Err(err) => {
                println!("Failed to find team Team J, with error: {:?}", err);
                panic!();
            }
        }
        TeamDao::delete(&mut conn, team_id).expect("Failed to remove the test record");
    }

    #[test]
    fn test_rotate_token() {

    }
}
