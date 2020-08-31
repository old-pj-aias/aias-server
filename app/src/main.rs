#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod handler;
pub mod utils;
pub mod tests;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::{Arc, Mutex};

use utils::{Keys};

use rusqlite::params;


use std::fs;


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let conn = utils::db_connection();

    if let Err(e) = conn.execute(
        "CREATE TABLE sign_process (
                  id              INTEGER PRIMARY KEY,
                  phone           TEXT NOT NULL,
                  m               TEXT NOT NULL,
                  subset          TEXT NOT NULL
                  )",
        params![],
    ) {
        eprintln!("error creating table: {}", e);
    }

    println!("server started");

    let signer_pubkey = fs::read_to_string("keys/signer_pubkey.pem")?;
    let signer_privkey = fs::read_to_string("keys/signer_privkey.pem")?;
    let judge_pubkey = fs::read_to_string("keys/judge_pubkey.pem")?;

    let data = Arc::new(Mutex::new(Keys {
        signer_pubkey: signer_pubkey,
        signer_privkey: signer_privkey,
        judge_pubkey: judge_pubkey
    }));

    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .route("/ready", web::post().to(handler::ready))
            .route("/sign", web::post().to(handler::sign))
            .route("/hello", web::get().to(handler::hello))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
