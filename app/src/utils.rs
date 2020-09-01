use rusqlite::{Connection, params, Result};
use actix_web::{HttpResponse};
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct Keys {
    pub signer_pubkey: String,
    pub signer_privkey: String
}

pub fn db_connection() -> Connection {
    Connection::open("db.sqlite3").unwrap()
}

pub fn create_table_sign_process() -> Result<()>{
    let conn = db_connection();
    conn.execute(
        "CREATE TABLE sign_process (
                  id              INTEGER PRIMARY KEY,
                  phone           TEXT NOT NULL,
                  blinded_digest  TEXT NOT NULL,
                  subset          TEXT NOT NULL,
                  session_id      INTEGER NOT NULL,
                  judge_pubkey    TEXT NOT NULL
                  )",
        params![],
    )?;

    Ok(())
}

pub fn parse_or_400<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T, HttpResponse> {
    serde_json::from_slice(&data.as_bytes())
        .map_err(bad_request)
}

pub fn bad_request<T: ToString>(data: T) -> HttpResponse {
    HttpResponse::BadRequest().body(data.to_string())
}