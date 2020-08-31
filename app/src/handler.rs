use actix_web::{HttpResponse, Responder, web};
use fair_blind_signature::CheckParameter;

use crate::*;

pub async fn hello() -> impl Responder {
    println!("hello");

    HttpResponse::Ok().body("Hello world")
}

pub async fn check(bytes: web::Bytes) -> Result<String, HttpResponse> {
    println!("check");

    let bytes = bytes.to_vec();
    let data = String::from_utf8_lossy(&bytes);
    let params: CheckParameter = get_body(&data)?;

    serde_json::to_string(&params).map_err(|e| HttpResponse::BadRequest().body(e.to_string()))
}
