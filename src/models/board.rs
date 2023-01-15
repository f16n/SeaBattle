use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct Board {
    pub game_id: u32,
    pub user_name: String,
    pub player_id: u8,
    pub status: u8,
    pub shots_fired: u16,
    pub shots_map: Vec<u8>,
    pub score: u16,
}

#[derive(Deserialize, Serialize, sqlx::Type, Debug)]
pub enum BoardStatus {
    Placing,
    Shooting,
    Waiting,
    Won,
    Lost,
}