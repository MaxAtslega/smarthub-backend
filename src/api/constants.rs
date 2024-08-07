use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::SqliteConnection;

use crate::api::{AppState, ErrorMessage, internal_error};
use crate::models::constants::{Constant, UpdateConstant};

pub async fn get_constants_by_user_id(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Constant>>, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let constants_result = Constant::get_all_by_user_id(id, &mut conn);

    match constants_result {
        Ok(constants) => Ok(Json(constants)),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to load constants".to_string() }))),
    }
}

pub async fn post_constant(
    State(state): State<Arc<AppState>>,
    Json(new_constant): Json<crate::models::constants::NewConstant>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    if constants_exists(&new_constant.user_id, &new_constant.name, &mut conn) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorMessage { message: "Constant with the same name already exists".to_string() })));
    }

    let constant_result = Constant::create(new_constant.user_id, &new_constant.name, &new_constant.value, &mut conn);

    match constant_result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to create constant".to_string() }))),
    }
}

pub async fn delete_constant_by_user_id_and_name(
    Path((id, constant_name)): Path<(i32, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let delete_result = Constant::delete_by_user_id_and_name(id, &constant_name, &mut conn);

    match delete_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to delete constant".to_string() }))),
    }
}

pub async fn put_constant(
    Path((id, constant_name)): Path<(i32, String)>,
    State(state): State<Arc<AppState>>,
    Json(new_value): Json<UpdateConstant>,
) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    let mut conn = state.db_pool.get().map_err(internal_error)?;

    let update_result = Constant::update_value(id, &constant_name, &new_value.value, &mut conn);

    match update_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { message: "Failed to update constant".to_string() }))),
    }
}

fn constants_exists(user_id: &i32, name: &String, conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>) -> bool {
    let constants_result = Constant::get_all_by_user_id_and_name(*user_id, name, conn);
    if let Ok(actions) = constants_result {
        if !actions.is_empty() {
            return true;
        }
    }
    false
}

