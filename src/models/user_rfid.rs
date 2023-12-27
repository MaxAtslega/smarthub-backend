use diesel::prelude::*;
use serde_derive::{Deserialize, Serialize};
use crate::schema::user_rfid;

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Deserialize, Serialize)]
#[table_name = "user_rfid"]
pub struct UserRfid {
    pub id: i32,
    pub rfid_uid: String,
    pub action_id: i32,
    pub created_on: chrono::NaiveDateTime,
}
