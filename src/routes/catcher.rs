use rocket::serde::{json::Json, Serialize};

#[derive(Serialize)]
pub struct ErrorResponse {
    code: i32,
    message: String,
}

#[catch(400)]
pub fn bad_request() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: "Bad Request".to_string(),
        code: 400,
    })
}

#[catch(422)]
pub fn unprocessable_entity() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: "Unprocessable Entity".to_string(),
        code: 422,
    })
}

#[catch(401)]
pub fn unauthorized() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: "Unauthorized".to_string(),
        code: 401,
    })
}

#[catch(403)]
pub fn forbidden() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: "Forbidden".to_string(),
        code: 403,
    })
}

#[catch(404)]
pub fn not_found() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: "Not found".to_string(),
        code: 404,
    })
}

#[catch(501)]
pub fn not_implemented() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: "Not Implemented".to_string(),
        code: 501,
    })
}

#[catch(500)]
pub fn internal_error() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: "Internal Server Error".to_string(),
        code: 500,
    })
}