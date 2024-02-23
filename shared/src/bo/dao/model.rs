use diesel::prelude::*;
use serde_json;
//use crate::artifact::ArtifactStatus;

use super::schema;

#[derive(Insertable)]
#[diesel(table_name=schema::artifact)]
pub struct NewArtifact {
    pub name: String
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name=schema::artifact)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Artifact {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub total: i32,
    pub target: i32,
    pub team_id: i32,
    pub build: serde_json::Value,
    pub clean: Option<serde_json::Value>
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name=schema::team)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Team {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub token: String,
    pub desp: Option<String>
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name=schema::account)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub total: i32,
    pub in_stock: i32,
    pub data: String,
    pub owner: Option<i32>,
    pub desp: Option<String>
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name=schema::secret)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Secret {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub data: String,
    pub owner: Option<i32>,
    pub desp: Option<String>
}
