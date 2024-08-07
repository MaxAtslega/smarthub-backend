use diesel::{RunQueryDsl, SqliteConnection};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use serde_derive::{Deserialize, Serialize};

use crate::schema::user_requests::dsl::*;

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::user_requests)]
pub struct UserRequest {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub endpoint: String,
    pub parameters: String,
    pub created_on: chrono::NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::user_requests)]
pub struct NewUserRequest {
    pub user_id: i32,
    pub name: String,
    pub endpoint: String,
    pub parameters: String,
}

#[derive(AsChangeset, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::user_requests)]
pub struct UserRequestChangeset {
    pub name: String,
    pub endpoint: String,
    pub parameters: String,
}

impl UserRequest {
    pub fn get_all_by_user_id(uid: i32, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Vec<UserRequest>, diesel::result::Error> {
        user_requests.filter(user_id.eq(uid)).load::<UserRequest>(conn)
    }

    pub fn get_all_by_user_id_and_name(uid: i32, constant_name: &String, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Vec<UserRequest>, diesel::result::Error> {
        user_requests.filter(user_id.eq(uid).and(name.eq(constant_name))).load::<UserRequest>(conn)
    }

    pub fn delete_by_id(aid: i32, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::delete(user_requests.filter(id.eq(aid))).execute(conn)
    }

    pub fn create(
        new_user_request: NewUserRequest,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(user_requests)
            .values(&new_user_request)
            .execute(conn)
    }

    pub fn update(request_id: i32,
                  changes: UserRequestChangeset,
                  conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(user_requests.filter(id.eq(request_id)))
            .set(changes)
            .execute(conn)
    }
}