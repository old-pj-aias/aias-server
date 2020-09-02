#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod handler;
pub mod utils;
pub mod tests;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_session::{CookieSession};

use std::sync::{Arc, Mutex};

use utils::{Keys};

use rusqlite::params;


use std::fs;


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    fs::remove_file("db.sqlite3").unwrap_or_else(|e| {
        eprintln!("an error occured on removing db data: {}", e);
    });

    let conn = utils::db_connection();
    utils::create_table_sign_process().unwrap_or_else(|e| {
        eprintln!("error creating table: {}", e);
    });


    println!("server started");

    let signer_pubkey = fs::read_to_string("keys/signer_pubkey.pem")?;
    let signer_privkey = fs::read_to_string("keys/signer_privkey.pem")?;
    let judge_pubkey = fs::read_to_string("keys/judge_pubkey.pem")?;

    let data = Arc::new(Mutex::new(Keys {
        signer_pubkey: signer_pubkey,
        signer_privkey: signer_privkey
    }));

    HttpServer::new(move || {
        App::new()
            .wrap(
                CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                    .secure(false)
            )
            .data(data.clone())
            .route("/send_sms", web::get().to(handler::send_sms))
            .route("/send_id", web::get().to(handler::send_id))
            .route("/ready", web::post().to(handler::ready))
            .route("/sign", web::post().to(handler::sign))
            .route("/hello", web::get().to(handler::hello))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
