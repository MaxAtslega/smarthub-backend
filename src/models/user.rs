use chrono::{NaiveDate, NaiveDateTime};
use diesel::{RunQueryDsl, SqliteConnection};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use serde_derive::{Deserialize, Serialize};

use crate::schema::user_users::dsl::*;

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::user_users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub theme: i32,
    pub birthday: NaiveDate,
    pub language: String,
    pub created_on: NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Deserialize, Serialize, Debug)]
#[diesel(table_name = crate::schema::user_users)]
pub struct NewUser {
    pub username: String,
    pub birthday: chrono::NaiveDate,
    pub theme: i32,
    pub language: String,
}

#[derive(AsChangeset, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::user_users)]
pub struct UserChangeset {
    pub username: Option<String>,
    pub birthday: Option<chrono::NaiveDate>,
    pub theme: Option<i32>,
    pub language: Option<String>,
}

impl User {
    pub fn all(conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Vec<User>, diesel::result::Error> {
        user_users.load::<User>(conn)
    }

    pub fn get_by_id(user_id: i32, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<User, diesel::result::Error> {
        user_users
            .filter(id.eq(user_id))
            .first::<User>(conn)
    }

    pub fn get_by_username(user_username: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<User, diesel::result::Error> {
        user_users
            .filter(username.eq(user_username))
            .first::<User>(conn)
    }

    pub fn new(
        new_user_data: NewUser,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(user_users)
            .values(&new_user_data)
            .execute(conn)
    }

    pub fn delete(user_id: i32, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::delete(user_users.filter(id.eq(user_id)))
            .execute(conn)
    }

    pub fn update(user_id: i32,
                  changes: UserChangeset,
                  conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(user_users.filter(id.eq(user_id)))
            .set(changes)
            .execute(conn)
    }

    pub fn set_theme(user_id: i32, user_theme: i32, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(user_users.filter(id.eq(user_id)))
            .set(theme.eq(user_theme))
            .execute(conn)
    }

    pub fn set_language(user_id: i32, user_language: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(user_users.filter(id.eq(user_id)))
            .set(language.eq(user_language))
            .execute(conn)
    }
}