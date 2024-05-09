use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use http::StatusCode;

use crate::api::{AppState, ErrorMessage, internal_error};
use crate::models::user::{NewUser, User, UserChangeset};

pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let users_result = User::all(&mut conn);

    match users_result {
        Ok(users) => Ok(Json(users)),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to load all users".to_string() }))),
    }
}

pub async fn get_user_by_id(
    Path(user_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<User>, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let user_result = User::get_by_id(user_id, &mut conn);

    match user_result {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err((StatusCode::NOT_FOUND, Json(ErrorMessage { message: "User not found".to_string() }))),
    }
}

pub async fn post_user(
    State(state): State<Arc<AppState>>,
    Json(new_user): Json<NewUser>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let existing_user = User::get_by_username(&new_user.username, &mut conn);
    if let Ok(_) = existing_user {
        return Err((StatusCode::FORBIDDEN, Json(ErrorMessage { message: "User with the same name already exists".to_string() })));
    }

    let user_result = User::new(new_user, &mut conn);
    match user_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to create user".to_string() }))),
    }
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let user_result = User::delete(user_id, &mut conn);
    match user_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to delete user".to_string() }))),
    }
}

pub async fn put_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Json(updated_user): Json<UserChangeset>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let user_result = User::update(user_id, updated_user, &mut conn);
    match user_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to update user".to_string() }))),
    }
}