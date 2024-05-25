use diesel::{RunQueryDsl, SqliteConnection};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use serde_derive::{Deserialize, Serialize};

use crate::schema::constants::dsl::*;

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::constants)]
pub struct Constant {
    pub id: i32,
    pub name: String,
    pub user_id: i32,
    pub value: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::constants)]
pub struct NewConstant {
    pub name: String,
    pub user_id: i32,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateConstant {
    pub value: String,
}

impl Constant {
    // Method to get all constants for a specific user_id
    pub fn get_all_by_user_id(uid: i32, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Vec<Constant>, diesel::result::Error> {
        constants.filter(user_id.eq(uid)).load::<Constant>(conn)
    }

    pub fn get_all_by_user_id_and_name(uid: i32, constant_name: &String, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Vec<Constant>, diesel::result::Error> {
        constants.filter(user_id.eq(uid).and(name.eq(constant_name))).load::<Constant>(conn)
    }

    // Method to delete a constant by user_id and name
    pub fn delete_by_user_id_and_name(uid: i32, constant_name: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::delete(constants.filter(user_id.eq(uid).and(name.eq(constant_name))))
            .execute(conn)
    }

    // Method to create a new constant with user_id, name, and value
    pub fn create(uid: i32, constant_name: &str, constant_value: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(constants)
            .values((user_id.eq(uid), name.eq(constant_name), value.eq(constant_value)))
            .execute(conn)
    }

    // Method to update the value of a constant by user_id and name
    pub fn update_value(uid: i32, constant_name: &str, new_value: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(constants.filter(user_id.eq(uid).and(name.eq(constant_name))))
            .set(value.eq(new_value))
            .execute(conn)
    }
}
