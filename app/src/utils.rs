use rusqlite::{params, Connection};
use serde::Deserialize;

use actix_web::{web, HttpResponse};

use twilio::{Client, OutboundMessage};

use std::env;

#[derive(Debug, Default)]
pub struct Keys {
    pub signer_pubkey: String,
    pub signer_privkey: String,
}

pub fn db_connection() -> Connection {
    Connection::open("db.sqlite3").unwrap()
}

pub fn create_table_sign_process() -> rusqlite::Result<()> {
    let conn = db_connection();
    conn.execute(
        "CREATE TABLE users (
                  id              INTEGER PRIMARY KEY,
                  phone           TEXT NOT NULL,
                  token           TEXT NOT NULL
                  )",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE sign_process (
                  id              INTEGER,
                  blinded_digest  TEXT NOT NULL,
                  subset          TEXT NOT NULL,
                  judge_pubkey    TEXT NOT NULL
                  )",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE sms_codes (
                  id              INTEGER,
                  secret_code     INTEGER
                  )",
        params![],
    )?;

    Ok(())
}

pub fn parse_or_400<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T, HttpResponse> {
    serde_json::from_slice(&data.as_bytes()).map_err(bad_request)
}

pub fn bad_request<T: ToString>(data: T) -> HttpResponse {
    let s = data.to_string();
    eprintln!("bad request: {}", s);
    HttpResponse::BadRequest().body(s)
}

pub fn internal_server_error<T: ToString>(data: T) -> HttpResponse {
    let s = data.to_string();
    eprintln!("internal server error: {}", s);
    HttpResponse::InternalServerError().body(s)
}

pub fn send_sms(to: String, body: String) {
    let from = env::var("FROM").expect("Find ACCOUNT_ID environment variable");
    let app_id = env::var("ACCOUNT_ID").expect("Find ACCOUNT_ID environment variable");
    let auth_token = env::var("AUTH_TOKEN").expect("Find AUTH_TOKEN environment variable");

    let client = Client::new(&app_id, &auth_token);
    let msg = OutboundMessage::new(&from, &to, &body);

    match client.send_message(msg) {
        Err(e) => println!("{:?}", e),
        Ok(m) => println!("{:?}", m),
    }
}

pub fn get_code(bytes: &web::Bytes) -> Result<u32, &'static str> {
    let data = String::from_utf8_lossy(&bytes).to_string();
    serde_json::from_str(&data).map_err(|_| "failed to parse json")
}
