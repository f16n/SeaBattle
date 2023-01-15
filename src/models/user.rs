use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub name: String,
    pub password_hash: String,
    pub display_name: String,
    pub email_address: String,
    pub admin: bool,
    pub active: bool,
    pub notify: bool,
    pub verification: u32,
    pub new_password_hash: String
}

// The struct used for receiving user data for creating a user record as json
#[derive(Deserialize, Serialize)]
pub struct NewUser {
    pub name: String,
    pub password: String,
    pub display_name: String,
    pub email_address: String,
    pub admin: bool,
    pub active: bool,
    pub notify: bool,
}

// The struct used for receiving user data for creating a user record as json
#[derive(Deserialize, Serialize)]
pub struct SignUp {
    pub name: String,
    pub password: String,
    pub display_name: String,
    pub email_address: String,
    pub notify: bool,
}
// The struct used for receiving user data for updating the user record as json
#[derive(Deserialize, Serialize)]
pub struct UpdateUser {
    pub display_name: String,
    pub email_address: String,
    pub admin: bool,
    pub active: bool,
    pub notify: bool,
}

// The struct used for receiving a the old and new password as json
#[derive(Deserialize, Serialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
}

// The struct used for receiving verification as json
#[derive(Deserialize, Serialize)]
pub struct Verification {
    pub verification_number: u32,
}

// The struct used to respond with an official json for the bearer token
#[derive(Deserialize, Serialize, Debug)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}