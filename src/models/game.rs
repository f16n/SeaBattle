use serde::{Deserialize, Serialize};
use chrono::Local;

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct Game {
    pub id: u32,
    pub status: u8,
    pub board_size: u8,
    pub amount_of_players: u8,
    pub placing: chrono::DateTime<Local>,
    pub started: chrono::DateTime<Local>,
    pub finished: chrono::DateTime<Local>,
}

#[derive(Deserialize, Serialize, sqlx::Type, Debug, PartialEq)]
pub enum GameStatus {
    Active,
    Finished,
    Aborted,
}
