use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{params, Connection, Result};
use rusqlite::NO_PARAMS;

use std::fs;

async fn hello() -> impl Responder {
    println!("request");

    HttpResponse::Ok().body("Hello world")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    fs::remove_file("db.sqlite3");
    let conn = Connection::open("db.sqlite3").unwrap();

    conn.execute(
        "CREATE TABLE sign_process (
                  id              INTEGER PRIMARY KEY,
                  phone           TEXT NOT NULL,
                  m               TEXT NOT NULL,
                  subset          TEXT NOT NULL
                  )",
        params![],
    ).unwrap();

    println!("server started");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
