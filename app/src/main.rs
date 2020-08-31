use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{params, Connection, Result};


fn db_connection() -> Connection {
    Connection::open("db.sqlite3").unwrap()
}

struct Data {
    data: String,
}

async fn hello() -> impl Responder {
    println!("request");

    let conn = db_connection();

    conn.execute("INSERT INTO sign_process (phone, m, subset)
                  VALUES ($1, $2, $3)",
                 &["000-000-0000", "this is m", "this is subset"]).unwrap();

    
    let mut stmt = conn.prepare("SELECT subset FROM sign_process WHERE id=?").unwrap();

    let subset = stmt.query_map(params![1], |row| {
        Ok(Data { data: row.get(0).unwrap() })
    }).unwrap();

    for s in subset {
        println!("m: {}", s.unwrap().data);
    }

    HttpResponse::Ok().body("Hello world")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let conn = db_connection();

    conn.execute(
        "CREATE TABLE sign_process (
                  id              INTEGER PRIMARY KEY,
                  phone           TEXT NOT NULL,
                  m               TEXT NOT NULL,
                  subset          TEXT NOT NULL
                  )",
        params![],
    );

    println!("server started");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
