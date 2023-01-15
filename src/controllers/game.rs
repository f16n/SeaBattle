use axum::{
    Extension, Json, response::IntoResponse,
    extract::{Path, TypedHeader, State},
    headers::{Authorization, authorization::Bearer},
    http::{StatusCode, header::{HeaderMap, HeaderValue, AUTHORIZATION}},
};
//use axum_macros::debug_handler;
use sqlx::{MySqlPool, mysql::MySqlQueryResult};
use log::{debug, error, log_enabled, info, Level};
use serde::{Deserialize, Serialize};
use chrono::Local;
use bit_vec::BitVec;
use crate::models::{game::*, board::*};
use crate::models::board;
use crate::errors::CustomError;

use crate::AppState;
use crate::check_access;

// The struct used for a new game
#[derive(Deserialize, Serialize, Debug)]
pub struct NewGame {
    boardSize: u8,
    players: u8,
}

//handler for creating a new game.
pub async fn new_game(  State(state): State<AppState>,
                        Extension(pool): Extension<MySqlPool>,
                        TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
                        Json(newgame): Json<NewGame>
                        ) -> Result<impl IntoResponse, CustomError> {

    info!("new game request");

    //check if user is logged in, bail out if not. Retrieve the user_name from the bearer token
    let (user_name, _) = check_access(&state, &bearer, false).await?;

    // check if board size is correct
    if newgame.boardSize < 8 || newgame.boardSize > 16 {
        info!("Illegal boardsize: {:?}", newgame.boardSize);
        return Err(CustomError::IllegalBoardSize);
    }

    // check if amount of players is correct
    if newgame.players < 2 || newgame.players > 4 {
        info!("Illegal amount of players: {:?}", newgame.players);
        return Err(CustomError::InvalidPlayers);
    }

    // Check if user is not DDOS-ing the server, max 3 Active games
    let sql = "SELECT board.game_id FROM board INNER JOIN game ON board.game_id=game.id WHERE game.status=? AND board.user_name=?";
    let active_games =  match sqlx::query(sql)
        .bind(GameStatus::Active)
        .bind(&user_name)
        .fetch_all(&pool)
        .await {
            Ok(result) => result,
            Err(err) => {
                error!("Error creating game: {:?}", err);
                return Err(CustomError::BadRequest);              
            }};

    if active_games.len() >= 3 {
        error!("Active games: {:?} for user {:?}, no more allowed", active_games.len(), user_name);
        return Err(CustomError::MaxGames);
    }

    // Start transaction
    let mut tx = match pool.begin()
        .await {
            Ok(tx) => tx,
            Err(err) => {
                error!("Error creating game: {:?}", err);
                return Err(CustomError::BadRequest);
            }
        };


    // Insert game. Most initital values are determined by the DB schema at create time. We want the game_id returned
    let sql = "INSERT INTO game (board_size, amount_of_players, status) VALUES (?, ?, ?)";

    let game_id = match sqlx::query(sql)
        .bind(newgame.boardSize)
        .bind(newgame.players)
        .bind(GameStatus::Active as u8)
        .execute(&mut tx)
        .await {
            Ok(result) => result.last_insert_id() as u32,
            Err(err) => {
                error!("Error creating game: {:?}", err);
                return Err(CustomError::BadRequest);              
            }};

    let shots_map = BitVec::from_elem((newgame.boardSize as usize) * (newgame.boardSize as usize), false).to_bytes();
    let player_id: u8 = 1;

    let sql = "INSERT INTO board (game_id, user_name, player_id, status, shots_map) VALUES (?, ?, ?, ?, ?)";

    // Execute the query using the provided data
    let _ = sqlx::query(sql)
            .bind(game_id)
            .bind(user_name)
            .bind(player_id)
            .bind(BoardStatus::Placing as u8)
            .bind(shots_map)
            .execute(&mut tx)
            .await
            .map_err(|err| {
                error!("Error creating game: {:?}", err);
                CustomError::BadRequest
            });

    // commit
    let _ = tx.commit().await
        .map_err(|err| {
            error!("Error creating game: {:?}", err);
            CustomError::BadRequest
        }
    );

    // Done
    Ok((StatusCode::OK,"Game started, place your ships"))

}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//handler for joining an existing game. 
pub async fn join_game( Path(game_id): Path<String>,
                        State(state): State<AppState>,
                        Extension(pool): Extension<MySqlPool>,
                        TypedHeader(bearer): TypedHeader<Authorization<Bearer>>
                        ) -> Result<impl IntoResponse, CustomError> {
    
    info!("Join game request");

    //check if user is logged in, bail out if not. Retrieve the user_name from the bearer token
    let (user_name, _) = check_access(&state, &bearer, false).await?;

    //Check if game exists and in the right status
    let sql = "SELECT * FROM game WHERE id = ?";
    let game: Game = match sqlx::query_as(sql)
        .bind(&game_id)
        .fetch_one(&pool)
        .await {
            Ok(result) => result,
            Err(err) => {
                error!("Error 1 joining game: {:?}", err);
                return Err(CustomError::BadRequest);              
            }
        };

    // Check if the game is active, bail out if not
    if game.status != GameStatus::Active as u8{
        return Err(CustomError::GameNotActive);
    }

    // Check if the user is not already in too many other games, if so, bail out
    let sql = "SELECT board.game_id FROM board INNER JOIN game ON board.game_id=game.id WHERE game.status=? AND board.user_name=?";
    let active_games = match sqlx::query(sql)
        .bind(GameStatus::Active as u8)
        .bind(&user_name)
        .fetch_all(&pool)
        .await {
            Ok(result) => result,
            Err(err) => {
                error!("Error 2 joining game: {:?}", err);
                return Err(CustomError::BadRequest);              
            }};

    if active_games.len() >= 3 {
        error!("Active games: {:?} for user {:?}, no more allowed", active_games.len(), user_name);
        return Err(CustomError::MaxGames);
    }

    //Determine the current amount of players for this game
    let sql = "SELECT * FROM board WHERE game_id=?";
    let boards = match sqlx::query_as::<_, board::Board>(sql)
        .bind(&game_id)
        .fetch_all(&pool)
        .await {
            Ok(result) => result,
            Err(err) => {
                error!("Error 3 joining game: {:?}", err);
                return Err(CustomError::BadRequest);              
            }
        };

    //check if the user is among the already joined players, if so, bail out
    for board in &boards {
        if board.user_name == user_name {
            error!("User {} is already a player in game {}", user_name, game_id);
            return Err(CustomError::InvalidGame);
        }
    }

    // Prepare new player (board)
    let shots_map = BitVec::from_elem((game.board_size as usize) * (game.board_size as usize), false).to_bytes();
    let player_id: u8 = boards.len() as u8 + 1;
        
    // Add board to game
    let sql = "INSERT INTO board (game_id, user_name, player_id, status, shots_map) VALUES (?, ?, ?, ?, ?)";

    // Execute the query using the provided data
    let _ = sqlx::query(sql)
            .bind(game_id)
            .bind(user_name)
            .bind(player_id)
            .bind(BoardStatus::Placing as u8)
            .bind(shots_map)
            .execute(&pool)
            .await
            .map_err(|err| {
                error!("Error 4 joining game: {:?}", err);
                CustomError::BadRequest
            });

    Ok((StatusCode::OK,"Game joined, place your ships"))
}