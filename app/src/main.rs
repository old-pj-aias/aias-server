use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
//use web::Json;

use serde_json;
use serde::{Deserialize};

use fair_blind_signature::CheckParameter;

#[get("/hello")]
async fn hello() -> impl Responder {
    println!("hello");

    HttpResponse::Ok().body("Hello world")
}

async fn check(bytes: web::Bytes) -> Result<String, HttpResponse> {
    println!("check");

    let bytes = bytes.to_vec();
    let data = String::from_utf8_lossy(&bytes);
    let params: CheckParameter = get_body(&data)?;

    serde_json::to_string(&params).map_err(|e| HttpResponse::BadRequest().body(e.to_string()))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("server started");

    HttpServer::new(|| {
        App::new()
            .route("/check", web::post().to(check))
            .service(hello)
    })
    .bind("localhost:8080")?
    .run()
    .await
}

fn get_body<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T, HttpResponse> {
    serde_json::from_slice(&data.as_bytes())
        .map_err(|e| HttpResponse::BadRequest().body(e.to_string()))
}