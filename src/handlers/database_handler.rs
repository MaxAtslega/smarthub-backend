use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;
use crate::common::db::DatabasePool;
use crate::models::user::{NewUser, User, UserChangeset};
use crate::models::websocket::WebSocketMessage;

pub async fn get_users(pool: &DatabasePool, sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>){
    let mut conn = pool.get().expect("Cannot get db connection from pool");

    let users = User::all(&mut conn);

    if users.is_err() {
        return;
    }

    let websocket = WebSocketMessage {
        op: 0,
        t: Some("USERS".to_string()),
        d: Some(json!(users.unwrap())),
    };

    if let Ok(json_msg) = serde_json::to_string(&websocket) {
        sender.send(Message::Text(json_msg)).await.expect("Failed to send message");
    }
}

pub async fn delete_user(pool: &DatabasePool, user_id: i32, sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>){
    let mut conn = pool.get().expect("Cannot get db connection from pool");

    let result = User::delete(user_id, &mut conn);

    if result.is_err() {
        return;
    }

    let websocket = WebSocketMessage {
        op: 0,
        t: Some("USER_DELETED".to_string()),
        d: Some(json!({
            "id": user_id
        })),
    };

    if let Ok(json_msg) = serde_json::to_string(&websocket) {
        sender.send(Message::Text(json_msg)).await.expect("Failed to send message");
    }
}

pub async fn create_user(
    pool: &DatabasePool,
    user: NewUser,
    sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>
) {
    let mut conn = pool.get().expect("Cannot get db connection from pool");

    let result = User::new(user, &mut conn);

    let websocket_msg = match result {
        Ok(_) => WebSocketMessage {
            op: 0,
            t: Some("USER_CREATED".to_string()),
            d: Some(json!({"status": "success"})),
        },
        Err(_) => WebSocketMessage {
            op: 0,
            t: Some("USER_CREATE_FAILED".to_string()),
            d: Some(json!({"status": "error"})),
        },
    };

    if let Ok(json_msg) = serde_json::to_string(&websocket_msg) {
        sender.send(Message::Text(json_msg)).await.expect("Failed to send message");
    }
}

pub async fn update_user(
    pool: &DatabasePool,
    user_id: i32,
    user_username: Option<String>,
    user_birthday: Option<Option<chrono::NaiveDate>>,
    user_theme: Option<i32>,
    user_language: Option<String>,
    sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>
) {
    let mut conn = pool.get().expect("Cannot get db connection from pool");

    let changes = UserChangeset {
        username: user_username.clone(),
        birthday: user_birthday.clone(),
        theme: user_theme,
        language: user_language.clone(),
    };

    let result = User::update(user_id, changes, &mut conn);

    let websocket_msg = match result {
        Ok(_) => WebSocketMessage {
            op: 0,
            t: Some("USER_UPDATED".to_string()),
            d: Some(json!({
                "id": user_id,
                "username": user_username,
                "birthday": user_birthday,
                "language": user_language,
            })),
        },
        Err(_) => WebSocketMessage {
            op: 0,
            t: Some("USER_UPDATE_FAILED".to_string()),
            d: Some(json!({"status": "error"})),
        },
    };

    if let Ok(json_msg) = serde_json::to_string(&websocket_msg) {
        sender.send(Message::Text(json_msg)).await.expect("Failed to send message");
    }
}




pub async fn get_contants(pool: &DatabasePool, sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>){
    let mut conn = pool.get().expect("Cannot get db connection from pool");

    let constants = crate::models::constants::Constant::get_all(&mut conn);

    if constants.is_err() {
        return;
    }

    let websocket = WebSocketMessage {
        op: 0,
        t: Some("CONSTANTS".to_string()),
        d: Some(json!(constants.unwrap())),
    };

    if let Ok(json_msg) = serde_json::to_string(&websocket) {
        sender.send(Message::Text(json_msg)).await.expect("Failed to send message");
    }
}

pub async fn set_constant(pool: &DatabasePool, constant_name: String, constant_value: String, sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>){
    let mut conn = pool.get().expect("Cannot get db connection from pool");

    let result = crate::models::constants::Constant::set_value(&constant_name, &constant_value, &mut conn);

    if result.is_err() {
        return;
    }

    let websocket = WebSocketMessage {
        op: 0,
        t: Some("CONSTANT_SET".to_string()),
        d: Some(json!({
            "name": constant_name,
            "value": constant_value
        })),
    };

    if let Ok(json_msg) = serde_json::to_string(&websocket) {
        sender.send(Message::Text(json_msg)).await.expect("Failed to send message");
    }
}

pub async fn create_constant(pool: &DatabasePool, constant_name: String, constant_value: String, sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>){
    let mut conn = pool.get().expect("Cannot get db connection from pool");

    let result = crate::models::constants::Constant::new(&constant_name, &constant_value, &mut conn);

    if result.is_err() {
        return;
    }

    let websocket = WebSocketMessage {
        op: 0,
        t: Some("CONSTANT_CREATED".to_string()),
        d: Some(json!({
            "name": constant_name,
            "value": constant_value
        })),
    };

    if let Ok(json_msg) = serde_json::to_string(&websocket) {
        sender.send(Message::Text(json_msg)).await.expect("Failed to send message");
    }
}

pub async fn delete_constant(pool: &DatabasePool, constant_name: String, sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>){
    let mut conn = pool.get().expect("Cannot get db connection from pool");

    let result = crate::models::constants::Constant::delete(&constant_name, &mut conn);

    if result.is_err() {
        return;
    }

    let websocket = WebSocketMessage {
        op: 0,
        t: Some("CONSTANT_DELETED".to_string()),
        d: Some(json!({
            "name": constant_name
        })),
    };

    if let Ok(json_msg) = serde_json::to_string(&websocket) {
        sender.send(Message::Text(json_msg)).await.expect("Failed to send message");
    }
}

