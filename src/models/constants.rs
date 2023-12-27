use serde_derive::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use crate::{schema::constants::{self, dsl::*}};

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[table_name = "constants"]
pub struct Constant {
    pub id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "constants"]
pub struct NewConstant {
    pub name: String,
    pub value: String,
}

impl Constant {
    pub fn get_value(constant_name: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<String, diesel::result::Error> {
        constants
            .filter(name.eq(constant_name))
            .select(value)
            .first::<String>(conn)
    }

    pub fn get_all(conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<Vec<Constant>, diesel::result::Error> {
        constants.load::<Constant>(conn)
    }

    pub fn set_value(constant_name: &str, constant_value: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(constants.filter(name.eq(constant_name)))
            .set(value.eq(constant_value))
            .execute(conn)
    }

    pub fn new(constant_name: &str, constant_value: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(constants)
            .values((name.eq(constant_name), value.eq(constant_value)))
            .execute(conn)
    }

    pub fn delete(constant_name: &str, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::delete(constants.filter(name.eq(constant_name)))
            .execute(conn)
    }
}
