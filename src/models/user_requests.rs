use diesel::prelude::*;
use serde_derive::{Deserialize, Serialize};
use crate::schema::user_requests;

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Deserialize, Serialize)]
#[table_name = "user_requests"]
pub struct UserRequest {
    pub id: i32,
    pub action_id: i32,
    pub endpoint: String,
    pub parameters: String,
    pub created_on: chrono::NaiveDateTime,
}
