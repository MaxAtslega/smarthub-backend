use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::SqliteConnection;

use crate::api::{AppState, ErrorMessage, internal_error};
use crate::models::user_actions::{NewUserAction, UserAction, UserActionChangeset};

pub async fn get_user_actions_by_user_id(
    Path(user_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserAction>>, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let user_actions_result = UserAction::get_all_by_user_id(user_id, &mut conn);

    match user_actions_result {
        Ok(user_actions) => Ok(Json(user_actions)),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to load user actions".to_string() }))),
    }
}

pub async fn post_user_action(
    State(state): State<Arc<AppState>>,
    Json(new_user_action): Json<NewUserAction>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    if user_action_exists(&new_user_action.rfid_uid, &mut conn) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorMessage { message: "Action with the same rfid already exists".to_string() })));
    }

    let create_result = UserAction::create(new_user_action, &mut conn);

    match create_result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to create user action".to_string() }))),
    }
}

pub async fn delete_user_action_by_id(
    Path(action_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let delete_result = UserAction::delete_by_id(action_id, &mut conn);

    match delete_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to delete user action".to_string() }))),
    }
}

pub async fn put_user_action(
    Path(action_id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Json(changes): Json<UserActionChangeset>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    if user_action_exists(&changes.rfid_uid, &mut conn) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorMessage { message: "Action with the same rfid already exists".to_string() })));
    }

    let update_result = UserAction::update(action_id, changes, &mut conn);

    match update_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to update user action".to_string() }))),
    }
}

fn user_action_exists(rfid_uid: &String, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> bool {
    let actions_result = UserAction::get_all_by_rfid_id(rfid_uid, conn);
    if let Ok(actions) = actions_result {
        if !actions.is_empty() {
            return true;
        }
    }
    false
}
