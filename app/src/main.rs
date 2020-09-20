pub mod handler;
pub mod utils;

#[cfg(test)]
mod tests;

use actix_session::CookieSession;
use actix_web::{web, App, HttpServer};

use std::sync::{Arc, Mutex};

use utils::Keys;

use std::fs;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    fs::remove_file("db.sqlite3").unwrap_or_else(|e| {
        eprintln!("an error occured on removing db data: {}", e);
    });

    utils::create_table_sign_process().unwrap_or_else(|e| {
        eprintln!("error creating table: {}", e);
    });

    println!("server started");

    let signer_pubkey = fs::read_to_string("keys/signer_pubkey.pem")?;
    let signer_privkey = fs::read_to_string("keys/signer_privkey.pem")?;

    let data = Arc::new(Mutex::new(Keys {
        signer_pubkey: signer_pubkey,
        signer_privkey: signer_privkey,
    }));

    HttpServer::new(move || {
        App::new()
            .wrap(
                CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                    .secure(false),
            )
            .data(data.clone())
            .route("/send_sms", web::post().to(handler::send_sms))
            .route("/verify_code", web::post().to(handler::verify_code))
            .route("/auth", web::post().to(handler::auth))
            .route("/ready", web::post().to(handler::ready))
            .route("/sign", web::post().to(handler::sign))
            .route("/hello", web::get().to(handler::hello))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
