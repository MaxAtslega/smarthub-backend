use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::SqliteConnection;

use crate::api::{AppState, ErrorMessage, internal_error};
use crate::models::user_requests::{NewUserRequest, UserRequest, UserRequestChangeset};

pub async fn get_user_requests_by_user_id(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserRequest>>, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let user_requests_result = UserRequest::get_all_by_user_id(id, &mut conn);

    match user_requests_result {
        Ok(user_requests) => Ok(Json(user_requests)),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to load user requests".to_string() }))),
    }
}

pub async fn post_user_request(
    State(state): State<Arc<AppState>>,
    Json(new_user_request): Json<NewUserRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    if action_exists(&new_user_request.user_id, &new_user_request.name, &mut conn) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorMessage { message: "Request with the same name already exists".to_string() })));
    }


    let create_result = UserRequest::create(new_user_request, &mut conn);

    match create_result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to create user request".to_string() }))),
    }
}

pub async fn delete_user_request_by_id(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let delete_result = UserRequest::delete_by_id(id, &mut conn);

    match delete_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to delete user request".to_string() }))),
    }
}

pub async fn put_user_request(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Json(changes): Json<UserRequestChangeset>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    if action_exists(&id, &changes.name, &mut conn) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorMessage { message: "Request with the same name already exists".to_string() })));
    }

    let update_result = UserRequest::update(id, changes, &mut conn);

    match update_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to update user request".to_string() }))),
    }
}

fn action_exists(user_id: &i32, name: &String, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> bool {
    let results = UserRequest::get_all_by_user_id_and_name(*user_id, name, conn);
    if let Ok(requests) = results {
        if !requests.is_empty() {
            return true;
        }
    }
    false
}
