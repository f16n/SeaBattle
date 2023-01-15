use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct Server {
    pub name: String,
    pub motd: String,
}
