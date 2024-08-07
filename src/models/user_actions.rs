use chrono::NaiveDateTime;
use diesel::{RunQueryDsl, SqliteConnection};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use serde_derive::{Deserialize, Serialize};

use crate::schema::user_actions::dsl::*;

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::user_actions)]
pub struct UserAction {
    pub id: i32,
    pub user_id: i32,
    pub rfid_uid: String,
    pub type_name: String,
    pub details: String,
    pub created_on: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::user_actions)]
pub struct NewUserAction {
    pub user_id: i32,
    pub rfid_uid: String,
    pub type_name: String,
    pub details: String,
}

#[derive(AsChangeset, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::user_actions)]
pub struct UserActionChangeset {
    pub user_id: i32,
    pub rfid_uid: String,
    pub type_name: String,
    pub details: String,
}

impl UserAction {
    // Method to get all user actions for a specific user_id
    pub fn get_all_by_user_id(uid: i32, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Vec<UserAction>, diesel::result::Error> {
        user_actions.filter(user_id.eq(uid)).load::<UserAction>(conn)
    }

    pub fn get_all_by_rfid_id(rfid: &String, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Vec<UserAction>, diesel::result::Error> {
        user_actions.filter(rfid_uid.eq(rfid)).load::<UserAction>(conn)
    }

    pub fn get_by_rfid_id(rfid: &String, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Option<UserAction>, diesel::result::Error> {
        user_actions.filter(rfid_uid.eq(rfid)).first(conn).optional()
    }

    pub fn delete_by_id(aid: i32, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::delete(user_actions.filter(id.eq(aid))).execute(conn)
    }

    // Method to create a new user_action with user_id, name, and value
    pub fn create(
        new_user_action: NewUserAction,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(user_actions)
            .values(&new_user_action)
            .execute(conn)
    }

    pub fn update(action_id: i32,
                  changes: UserActionChangeset,
                  conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(user_actions.filter(id.eq(action_id)))
            .set(changes)
            .execute(conn)
    }
}
