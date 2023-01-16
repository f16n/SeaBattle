use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

// Custom Errors used in handlers
pub enum CustomError {
    BadRequest,
    UserNotFound,
    UserExists,
    EmailExists,
    UserDeactivated,
    WrongPassword,
    NotAdmin,
    InternalServerError,
    InvalidToken,
    MaxGames,
    IllegalBoardSize,
    InvalidPlayers,
    GameNotActive,
    InvalidGame,
    VerificationFailure,
}

//implementation of custom errors that are used in handlers
impl IntoResponse for CustomError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            Self::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR,"Internal Server Error"),
            Self::BadRequest => (StatusCode::BAD_REQUEST, "Bad Request"),
            Self::UserNotFound => (StatusCode::NOT_FOUND, "User not Found"),
            Self::UserExists => (StatusCode::BAD_REQUEST, "User already exists"),
            Self::EmailExists => (StatusCode::BAD_REQUEST, "You already have an account"),
            Self::UserDeactivated => (StatusCode::BAD_REQUEST, "User deactivated"),
            Self::WrongPassword => (StatusCode::UNAUTHORIZED, "Wrong Password"),
            Self::NotAdmin => (StatusCode::UNAUTHORIZED, "You need to be an administrator for this request"),
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, "Token is not valid"),
            Self::MaxGames => (StatusCode::TOO_MANY_REQUESTS, "You already have the maximum amount of active games on this server"),
            Self::IllegalBoardSize => (StatusCode::BAD_REQUEST, "Board Size must be between 8 and 16"),
            Self::InvalidPlayers => (StatusCode::BAD_REQUEST, "Number of players must be between 2 and 4"),
            Self::GameNotActive => (StatusCode::BAD_REQUEST, "Game is not active"),
            Self::InvalidGame => (StatusCode::BAD_REQUEST, "Invalid Game"),
            Self::VerificationFailure => (StatusCode::BAD_REQUEST, "Verification failed"),
        };
        (status, Json(json!({"error": error_message}))).into_response()
    }
}


