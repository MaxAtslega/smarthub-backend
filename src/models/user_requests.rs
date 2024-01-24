use diesel::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::user_requests)]
pub struct UserRequest {
    pub id: i32,
    pub action_id: i32,
    pub endpoint: String,
    pub parameters: String,
    pub created_on: chrono::NaiveDateTime,
}
