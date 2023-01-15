use axum::{
    Extension, Json, response::IntoResponse,
    extract::{Path, TypedHeader, State},
    headers::{Authorization, authorization::Bearer},
    http::{StatusCode, header::{HeaderMap, HeaderValue, AUTHORIZATION}},
};
//use axum_macros::debug_handler;
use sqlx::MySqlPool;
use log::{debug, error, log_enabled, info, Level};
use serde::{Deserialize, Serialize};
use crate::models::server;
use crate::errors::CustomError;

use crate::AppState;
use crate::check_access;

#[derive(Deserialize, Serialize, Debug)]
pub struct Motd {
    pub motd: String
}

//handler for getting the motd. this request can be done without any auth
pub async fn get_motd(  State(_state): State<AppState>,
                        Extension(pool): Extension<MySqlPool>
                        ) -> Result<impl IntoResponse, CustomError> {

    info!("motd request");

    // Fetch the server info
    let sql = "SELECT * FROM server";
    match sqlx::query_as::<_, server::Server>(sql)
        .fetch_one(&pool)
        .await {
            Ok(server) => Ok((StatusCode::OK, Json(Motd{motd: server.motd}))),
            Err(err) => {
                error!("Unexpected error fetching server info. Error: {:?}", err);
                Err(CustomError::InternalServerError)       
            }
    }
}

// Handler for setting the motd. First check if the user doing this has an admin Bearer token.
pub async fn set_motd(  State(state): State<AppState>,
                        Extension(pool): Extension<MySqlPool>,
                        TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
                        Json(motd): Json<Motd>,
                        ) -> Result <(StatusCode,String), CustomError> {

    info!("Change MOTD request");

    //check if user is logged in and has the mandatory ADMIN role, bail out if not
    check_access(&state, &bearer, true).await?;

    // Change the MOTD
    let sql = "UPDATE server set motd=? WHERE name='battleship'";
    match sqlx::query(sql)
        .bind(&motd.motd)
        .execute(&pool)
        .await {
            Ok(_) => Ok((StatusCode::OK, "MOTD changed".to_string())),
            Err(err) => {
                error!("Error changing MOTD: {:?}", err);
                Err(CustomError::BadRequest)
            }
    }
}