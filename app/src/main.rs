use actix_web::{App, HttpServer, web};

use app::*;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("server started");

    HttpServer::new(|| {
        App::new()
            .route("/check", web::post().to(handler::check))
            .route("/hello", web::get().to(handler::hello))
    })
    .bind("localhost:8080")?
    .run()
    .await
}