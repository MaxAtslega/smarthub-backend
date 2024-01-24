use diesel::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::user_rfid)]
pub struct UserRfid {
    pub id: i32,
    pub rfid_uid: String,
    pub action_id: i32,
    pub created_on: chrono::NaiveDateTime,
}
