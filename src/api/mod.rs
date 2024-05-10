use std::sync::Arc;

use axum::{extract::{
    State,
    ws::WebSocketUpgrade,
}, Json, response::IntoResponse, Router, routing::get};
use axum::routing::{delete, post, put};
use http::StatusCode;
use log::info;
use serde_derive::Serialize;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tower_http::cors::CorsLayer;

use crate::api::actions::{delete_user_action_by_id, get_user_actions_by_user_id, post_user_action, put_user_action};
use crate::api::constants::{delete_constant_by_user_id_and_name, get_constants_by_user_id, post_constant, put_constant};
use crate::api::requests::{delete_user_request_by_id, get_user_requests_by_user_id, post_user_request, put_user_request};
use crate::api::system::{get_info, post_reboot, post_shutdown};
use crate::api::users::{delete_user, get_user_by_id, get_users, post_user, put_user};
use crate::common::db::DatabasePool;
use crate::config::ServerConf;
use crate::enums::system_command::SystemCommand;
use crate::handlers::connection_handler::handle_connection;
use crate::models::websocket::WebSocketMessage;

mod system;
mod users;
mod constants;
mod actions;
mod requests;

pub struct AppState {
    pub tx: broadcast::Sender<WebSocketMessage>,
    pub tx_dbus: Sender<SystemCommand>,
    pub db_pool: DatabasePool,
}

#[derive(Serialize)]
pub struct ErrorMessage {
    pub message: String,
}

pub async fn init(web_socket_conf: &ServerConf, tx: broadcast::Sender<WebSocketMessage>, tx_dbus: Sender<SystemCommand>, db_pool: &DatabasePool) {
    let address = format!("{}:{}", web_socket_conf.address, web_socket_conf.port);

    let app_state = Arc::new(AppState { tx, tx_dbus, db_pool: db_pool.clone() });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/", get(get_info))
        .route("/system/reboot", post(post_reboot))
        .route("/system/shutdown", post(post_shutdown))
        .route("/users", get(get_users))
        .route("/users", post(post_user))
        .route("/users/:user_id", get(get_user_by_id))
        .route("/users/:user_id", delete(delete_user))
        .route("/users/:user_id", put(put_user))
        .route("/constants/:user_id", get(get_constants_by_user_id))
        .route("/constants", post(post_constant))
        .route("/constants/:user_id/:constant_name", delete(delete_constant_by_user_id_and_name))
        .route("/constants/:user_id/:constant_name", put(put_constant))
        .route("/actions/:id", get(get_user_actions_by_user_id))
        .route("/actions", post(post_user_action))
        .route("/actions/:id", delete(delete_user_action_by_id))
        .route("/actions/:id", put(put_user_action))
        .route("/requests/:id", get(get_user_requests_by_user_id))
        .route("/requests", post(post_user_request))
        .route("/requests/:id", delete(delete_user_request_by_id))
        .route("/requests/:id", put(put_user_request))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let try_socket = TcpListener::bind(&address).await;

    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", address);

    axum::serve(listener, app).await.unwrap();
}


async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_connection(socket, state))
}

pub fn internal_error<E>(err: E) -> (StatusCode, Json<ErrorMessage>) where E: std::error::Error, {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: err.to_string() }))
}

