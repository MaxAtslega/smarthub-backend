use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DatabasePool = Pool<ConnectionManager<SqliteConnection>>;

pub fn establish_connection_pool(database_url: &str) -> DatabasePool {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);

    Pool::builder().build(manager).expect("Failed to create pool.")
}