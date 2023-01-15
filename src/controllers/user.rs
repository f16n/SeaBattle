use axum::{
    Extension, Json, response::{IntoResponse,Response},
    extract::{Path, TypedHeader, State},
    headers::{Authorization, authorization::{ Basic, Bearer}},
    http::StatusCode,
};
//use axum_macros::debug_handler;
use sqlx::MySqlPool;
use log::{debug, error, log_enabled, info, Level};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, TokenData};
use pwhash::bcrypt;
use rand;
use std::env;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};

use crate::models::user::*;
use crate::errors::CustomError;
use crate::Claims;
use crate::check_access;
use crate::AppState;

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//handler logging in. We extract Basic authentication to retrieve username and password from db. If password
//checks out we generate and return the JWT Bearer token which has the expiration and role encoded within
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn login( State(state): State<AppState>,
                    Extension(pool): Extension<MySqlPool>,
                    TypedHeader(basic): TypedHeader<Authorization<Basic>>
                    ) -> Result<impl IntoResponse, CustomError> {

    info!("login request by user: {}",basic.username());

    // Fetch the user using the username from the basic authentication header
    let sql = "SELECT * FROM user WHERE name = ?";
    let user: User = sqlx::query_as(sql)
        .bind(basic.username())
        .fetch_one(&pool)
        .await
        .map_err(|err| {
            error!("error retrieving user: {:?}", err);
            CustomError::UserNotFound
        })?;

    // Check if the user is active, if not, error out
    if ! user.active {
         Err(CustomError::UserDeactivated)?;
    }

    //Check password hash is equal to stored password hash. if not, error out
    if ! pwhash::bcrypt::verify(basic.password(), &user.password_hash) {
        Err(CustomError::WrongPassword)?;
    }

    // Define the registered <Expiration Time> claim (exp) which is the current timestmap plus the defined offset
    let my_exp = Utc::now()
        .checked_add_signed(Duration::seconds(state.token_duration))
        .expect("invalid timestamp")
        .timestamp();

    // Define the Claims struct
    let my_claims = Claims {
        sub: basic.username().to_string(),          // username
        iat: Utc::now().timestamp() as usize,       // valid from
        exp: my_exp as usize,                       // valid until
        admin: user.admin,                          // user role
    };

    // generate the Bearer token
    match encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes())
    ) {
        Ok(token) => {
            debug!("Generated token: {token}\n");
            Ok((StatusCode::OK, Json(AuthResponse{access_token: token, token_type: "bearer".to_string(), expires_in: state.token_duration})))
        }
        Err(err) => {
            error!("Unexpected error while encoding the bearer token ({:?})", err);
            Err(CustomError::InternalServerError)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Handler for creating a new user as adminsistrator.
// All fields are determind by the admin without any rules
// Password is hashed
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn new_user(  State(state): State<AppState>,
                        Extension(pool): Extension<MySqlPool>,
                        TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
                        Json(user): Json<NewUser>,
                        ) -> Result <(StatusCode,String), CustomError> {

    info!("new user request");

    //check if user is logged in and has the mandatory ADMIN role, bail out if not
    check_access(&state, &bearer, true).await?;

    //Create the password hash
    let password_hash = match bcrypt::hash(user.password) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Unexpected error encrypting password {:?}", err);
            return Err(CustomError::InternalServerError);
        }
    };

    // Create user
    let sql = "INSERT INTO user (name, password_hash, display_name, email_address, notify) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
    match sqlx::query(sql)
        .bind(user.name)
        .bind(password_hash)
        .bind(user.display_name)
        .bind(user.email_address)
        .bind(user.admin)
        .bind(user.active)
        .bind(user.notify)
        .execute(&pool)
        .await {
            Ok(_) => Ok((StatusCode::CREATED, "User added".to_string())),
            Err(err) => {
                error!("Error creating user: {:?}", err);
                Err(CustomError::BadRequest)
            }
        }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Handler for signing up. This means email verification is done and it's possible
// to create another admin and/or make the user acitve or not.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn signup(    State(state): State<AppState>,
                        Extension(pool): Extension<MySqlPool>,
                        Json(user): Json<SignUp>,
                        ) -> Result <(StatusCode,String), CustomError> {

    info!("signup request");

    // check if user already exists, bail out if that is the case
    let sql = "SELECT * FROM user where name=?";
    if sqlx::query_as::<_,User>(sql)
        .bind(&user.name)
        .fetch_one(&pool)
        .await.is_ok() {
                error!("Trying to signup with a username that already exists");
                return Err(CustomError::UserExists);
        }

    // Create a random verification number
    let verification_number: u32 = rand::random();

    // Create the password hash
    let password_hash = match bcrypt::hash(user.password) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Unexpected error encrypting password {:?}", err);
            return Err(CustomError::InternalServerError);
        }
    };

    mail_verification_code(&user.display_name, &user.email_address, &verification_number).await?;

    // Create user
    let sql = "INSERT INTO user (name, display_name, email_address, notify, verification, new_password_hash) VALUES (?, ?, ?, ?, ?, ?)";
    match sqlx::query(sql)
        .bind(user.name)
        .bind(user.display_name)
        .bind(user.email_address)
        .bind(user.notify)
        .bind(verification_number)
        .bind(password_hash)
        .execute(&pool)
        .await {
            Ok(_) => Ok((StatusCode::CREATED, "User added, waiting on verification".to_string())),
            Err(err) => {
                error!("Error creating user: {:?}", err);
                Err(CustomError::BadRequest)
            }
        }

}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Handler for verifying a signup. user must authenticate using basic authentication as there is no bearer token
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn signup_verification(   State(state): State<AppState>,
                                    Extension(pool): Extension<MySqlPool>,
                                    TypedHeader(basic): TypedHeader<Authorization<Basic>>,
                                    Json(verification): Json<Verification>,
                                    ) -> Result <(StatusCode,String), CustomError> {

    info!("signup verification request");

    // get user using the id
    let sql = "SELECT * FROM user where name=?";
    let user = match sqlx::query_as::<_,User>(sql)
        .bind(basic.username())
        .fetch_one(&pool)
        .await {
            Ok(user) => user,
            Err(err) => {
                error!("Error looking up user for password signup verification: {:?}", err);
                return Err(CustomError::InternalServerError);
            }
    };

    //Check if verification from request is equal to the stored verification 
    if verification.verification_number != user.verification {
        error!("Signup verification failed");
        return Err(CustomError::VerificationFailure);
    }

    //Everything checks out, active user
    let sql = "UPDATE user set password_hash=?, new_password_hash='', verification=0, active=true where name=?";
    match sqlx::query(sql)
        .bind(user.new_password_hash)
        .bind(basic.username())
        .execute(&pool)
        .await {
            Ok(_) => Ok((StatusCode::OK, "Signup verified and activated".to_string())),
            Err(err) => {
                error!("Error updating user: {:?}", err);
                Err(CustomError::BadRequest)
            }
    }

}


///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Handler for looking up a user.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn get_user(  Path(id): Path<String>, State(state): State<AppState>,
                        Extension(pool): Extension<MySqlPool>,
                        TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
                        ) -> Result <impl IntoResponse, CustomError> {

    info!("get user request");

    //check if user is logged in and has the mandatory ADMIN role, bail out if not
    check_access(&state, &bearer, true).await?;

    // get user using the id
    let sql = "SELECT * FROM user where name=?";
    match sqlx::query_as::<_,User>(sql)
        .bind(id)
        .fetch_one(&pool)
        .await {
            Ok(user) => Ok((StatusCode::OK,Json(user))),
            Err(_) => Err(CustomError::UserNotFound)
    }
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Handler for updating a user.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn update_user(   Path(id): Path<String>, State(state): State<AppState>,
                            Extension(pool): Extension<MySqlPool>,
                            TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
                            Json(updateUser): Json<UpdateUser>,
                            ) -> Result <impl IntoResponse, CustomError> {

    info!("Update user request");

    //check if user is logged in and has the mandatory ADMIN role, bail out if not
    check_access(&state, &bearer, true).await?;

    // Fetch the user using the username from the basic authentication header
    let sql = "SELECT * FROM user WHERE name = ?";
    let _: User = sqlx::query_as(sql)
        .bind(&id)
        .fetch_one(&pool)
        .await
        .map_err(|_| {
            CustomError::UserNotFound
        })?;

    // Update user
    let sql = "UPDATE user set display_name=?, email_address=?, admin=?, active=?, notify=? WHERE name = ?";
    match sqlx::query(sql)
        .bind(updateUser.display_name)
        .bind(updateUser.email_address)
        .bind(updateUser.admin)
        .bind(updateUser.active)
        .bind(updateUser.notify)
        .bind(&id)
        .execute(&pool)
        .await {
            Ok(_) => Ok((StatusCode::OK, "User updated".to_string())),
            Err(err) => {
                error!("Error updating user: {:?}", err);
                Err(CustomError::BadRequest)
            }
    }

    //TODO if user.name is changed (not equal to id) then also update user.name in table <boards>

}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Handler for changing a password.
// When logged in user is admin it is possible to change password for another user
// When logged in user is not admin it must be the same user as in the bearer token
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn change_password(   Path(id): Path<String>, State(state): State<AppState>,
                                Extension(pool): Extension<MySqlPool>,
                                TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
                                Json(passwords): Json<ChangePassword>,
                                ) -> Result <impl IntoResponse, CustomError> {

    info!("Change password request");

    //check if user is logged in, bail out if not, retrieve the user and admin status from the token
    let (token_user, token_admin) = check_access(&state, &bearer, false).await?;

    // You can only change someone else password if you are an admin. If not, bail out
    if !token_admin && token_user != id {
        return Err(CustomError::NotAdmin)
    }

    // get user using the id
    let sql = "SELECT * FROM user where name=?";
    let user = match sqlx::query_as::<_,User>(sql)
        .bind(&id)
        .fetch_one(&pool)
        .await {
            Ok(user) => user,
            Err(err) => {
                error!("Error looking up user for password change: {:?}", err);
                return Err(CustomError::InternalServerError);
            }
    };
    
    if ! bcrypt::verify(passwords.old_password, &user.password_hash) {
    // check if old password is equal to the stored password hash. If not, bail out
        error!("Password verification failed while changing password");
        return Err(CustomError::WrongPassword);
    }

    //Create the password hash of the new password
    let new_password_hash = match bcrypt::hash(passwords.new_password) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Unexpected error encrypting password {:?}", err);
            return Err(CustomError::InternalServerError);
        }
    };

    
    //ADMIN: Change new password_hash
    //USER: Store new password_hash in new_password_hash and verification_number in verification
    if token_admin {

        let sql = "UPDATE user set password_hash=? WHERE name = ?";
        match sqlx::query(sql)
            .bind(new_password_hash)
            .bind(id)
            .execute(&pool)
            .await {
                Ok(_) => Ok((StatusCode::OK, "Password changed".to_string())),
                Err(err) => {
                    error!("Error 1 changing password: {:?}", err);
                    Err(CustomError::BadRequest)
                }
        }
    } else {

        let verification_number: u32 = rand::random();

        mail_verification_code(&user.display_name, &user.email_address, &verification_number).await?;

        let sql = "UPDATE user set verification=?, new_password_hash=? WHERE name = ?";
        match sqlx::query(sql)
            .bind(verification_number)
            .bind(new_password_hash)
            .bind(id)
            .execute(&pool)
            .await {
                Ok(_) => Ok((StatusCode::OK, "Please verify your password change request".to_string())),
                Err(err) => {
                    error!("Error 2 changing password: {:?}", err);
                    Err(CustomError::BadRequest)
                }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Handler for verifying a password change.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn password_verification( State(state): State<AppState>,
                                    Extension(pool): Extension<MySqlPool>,
                                    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
                                    Json(verification): Json<Verification>,
                                    ) -> Result <(StatusCode,String), CustomError> {

    info!("password change verification request");

    //check if user is logged in and has the mandatory ADMIN role, bail out if not
    let (user_name, _) = check_access(&state, &bearer, false).await?;

    // get user using the id
    let sql = "SELECT * FROM user where name=?";
    let user = match sqlx::query_as::<_,User>(sql)
        .bind(user_name)
        .fetch_one(&pool)
        .await {
            Ok(user) => user,
            Err(err) => {
                error!("Error looking up user for password signup verification: {:?}", err);
                return Err(CustomError::InternalServerError);
            }
    };

    //Check if verification from request is equal to the stored verification 
    if verification.verification_number != user.verification {
        error!("Signup verification failed");
        return Err(CustomError::VerificationFailure);
    }

    //Everything checks out, active user
    let sql = "UPDATE user set password_hash=new_password_hash, new_password_hash='', verification=0, active=true";
    match sqlx::query(sql)
        .execute(&pool)
        .await {
            Ok(_) => Ok((StatusCode::OK, "Password change request verified. Password changed".to_string())),
            Err(err) => {
                error!("Error updating user: {:?}", err);
                return Err(CustomError::BadRequest);
            }
        }   
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Send verification mail message 
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn mail_verification_code(display_name: &String, email_address: &String, verification_number: &u32) -> Result<(),CustomError> {

    // Get the mail settings from environment variables
    let email_from = env::var("EMAIL_FROM").expect("$EMAIL_FROM is not set");
    let email_reply_to_name = env::var("EMAIL_REPLY_TO_NAME").expect("$EMAIL_REPLY_TO_NAME is not set");
    let email_reply_to_address = env::var("EMAIL_REPLY_TO_ADDRESS").expect("$EMAIL_REPLY_TO_ADDRESS is not set");
    let smtp_username = env::var("SMTP_USERNAME").expect("$SMTP_USERNAME is not set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("$SMTP_PASSWORD is not set");
    let smtp_host = env::var("SMTP_HOST").expect("$SMTP_HOST is not set");

    // Construct the mail message
    let email = match Message::builder()
        .from(email_from.parse().unwrap())
        .reply_to(format!("{} <{}>", email_reply_to_name, email_reply_to_address).parse().unwrap())
        .to(format!("{} <{}>", display_name, email_address).parse().unwrap())
        .subject("Here is your verification code for the requested action on the See Battle server")
        .body(String::from(format!("Verification number: {}", verification_number))) {
            Ok(message) => message,
            Err(err) => {
                error!("Error building mail message: {:?}", err);
                return Err(CustomError::InternalServerError);
            }
        };


    // Open a remote connection using STARTTLS
    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(email).await {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("Error while sending mail: {:?}", err);
            Err(CustomError::InternalServerError)
        }
    }

}

