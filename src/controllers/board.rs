use std::fmt::Result;

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
use chrono::Utc;
use bit_vec::BitVec;
use crate::models::board::*;
use crate::errors::CustomError;

use crate::AppState;
use crate::check_access;

