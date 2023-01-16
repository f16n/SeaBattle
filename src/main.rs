use axum::{ extract::{Extension},routing::{get, post, put, delete}, Router, 
    headers::{Authorization, authorization::Bearer}
};
use sqlx::{mysql::{MySqlPool, MySqlConnectOptions, MySqlPoolOptions, MySqlArguments}, ConnectOptions};
use std::{env, net::SocketAddr};
use log::{debug, error, log_enabled, info, Level};
use serde::{Deserialize, Serialize};
use simplelog::*;
mod errors;
mod controllers;
mod models;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, TokenData};
use crate::errors::CustomError;

// The claims struct used for creating a Bearer token
#[derive(Deserialize, Serialize, Debug)]
struct Claims {
    sub: String,
    iat: usize,
    exp: usize,
    admin: bool,
}

// Shared immutable state
#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
    pub token_duration: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // set up tracing facility
    let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());
    info!("Starting..");

    // get database url
    let database_url = env::var("DATABASE_URL").expect("$DATABASE_URL is not set");
    debug!("database_url: {:?}", database_url);

    //TODO: Disable statement logging
    //let pool = MySqlConnectOptions::new()
     //   .disable_statement_logging()
     //   .connect()
     //   .await?;
    
    let pool = MySqlPool::connect(&database_url).await?;

    // Retrieve the JWT secret and token duration from the env var and store it in the shared AppState
    let state = AppState {
        jwt_secret: env::var("JWT_SECRET").expect("$JWT_SECRET is not set"),
        token_duration: env::var("TOKEN_DURATION").expect("$TOKEN_DURATION is not set")
            .parse::<i64>().expect("$TOKEN_DURATION is not numeric")
    };

    // Define routes
    let app = Router::new()
        .route("/login", get(controllers::user::login))
        .route("/signup", post(controllers::user::signup))
        .route("/signup/verification", post(controllers::user::signup_verification))
        .route("/user", post(controllers::user::new_user))
        .route("/user/:id", get(controllers::user::get_user).post(controllers::user::update_user))
        .route("/user/:id/password", put(controllers::user::change_password))
        .route("/user/:id/verification", post(controllers::user::password_verification))
        .route("/motd",get(controllers::server::get_motd))
        .route("/server/motd", post(controllers::server::set_motd))
        .route("/game", post(controllers::game::new_game))
        .route("/game/:game_id", post(controllers::game::join_game))
        .with_state(state)
        .layer(Extension(pool));

    // Start the server
    // TODO: tls -> there is now a axum-server crate that does this
    // TODO: make duration also an env and remove state from all handlers (if possible)
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())

}

// Helper function to check if a bearer token is valid (user is logged in) and if the token is an admin token if needed.
// The JWT secret is retrieved from the state shared across all handlers (fetched from an env in main)
// It's here because basically every controller function needs it.
async fn check_access(state: &AppState, bearer: &Authorization<Bearer>, admin_needed: bool) -> Result<(String, bool),CustomError> {

    // Decode the Bearer token from the header. When succesfull return decoded user_name (sub field)
    match decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            if admin_needed && !token_data.claims.admin {
                error!("User is not admin, request denied");
                Err(CustomError::NotAdmin)
            } else {
                Ok((token_data.claims.sub, token_data.claims.admin))
            }
        },
        Err(err) => {
            error!("Invalid token: {:?}", err.kind());
            Err(CustomError::InvalidToken)
        }
    }
}