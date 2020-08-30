use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn hello() -> impl Responder {
    println!("request");

    HttpResponse::Ok().body("Hello world")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("server started");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
