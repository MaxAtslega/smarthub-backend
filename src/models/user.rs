use chrono::NaiveDateTime;
use serde_derive::Deserialize;

#[derive(Debug, Queryable, Deserialize)]
pub struct User {
    pub id: i32,

    pub first_name: String,
    pub last_name: String,

    pub birthday: NaiveDateTime,

    pub theme: i32,
    pub language: String,

    pub created_on: NaiveDateTime,
}