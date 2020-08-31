pub mod handler;
pub mod tests;

use rusqlite::{Connection, Result};
use actix_web::{HttpResponse};
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct Keys {
    pub signer_pubkey: String,
    pub signer_privkey: String,
    pub judge_pubkey: String
}

pub fn db_connection() -> Connection {
    Connection::open("db.sqlite3").unwrap()
}


pub fn get_body<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T, HttpResponse> {
    serde_json::from_slice(&data.as_bytes())
        .map_err(|e| HttpResponse::BadRequest().body(e.to_string()))
}