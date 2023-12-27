use diesel::prelude::*;
use serde_derive::{Deserialize, Serialize};
use crate::schema::user_actions;

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Deserialize, Serialize)]
#[table_name = "user_actions"]
pub struct UserAction {
    pub id: i32,
    pub user_id: i32,
    pub type_name: String,
    pub details: String,
    pub created_on: chrono::NaiveDateTime,
}
