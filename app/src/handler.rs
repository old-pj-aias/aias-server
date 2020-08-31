use actix_web::{HttpResponse, Responder, web};
use fair_blind_signature::CheckParameter;

use aias_core::signer::Signer;
use std::sync::{Arc, Mutex};


use crate::*;

pub async fn hello() -> impl Responder {
    println!("hello");
    
    HttpResponse::Ok().body("Hello world")
}


pub async fn ready(body: web::Bytes, actix_data: web::Data<Arc<Mutex<Keys>>>) -> Result<String, HttpResponse> {
    println!("ready");

    let body = String::from_utf8_lossy(&body);

    let signer_privkey = actix_data.lock().unwrap().signer_privkey.clone();
    let signer_pubkey = actix_data.lock().unwrap().signer_pubkey.clone();
    let judge_pubkey = actix_data.lock().unwrap().judge_pubkey.clone();

    let mut signer = Signer::new(signer_privkey, signer_pubkey, judge_pubkey);

    signer.set_blinded_digest(body.to_string()).unwrap();
    let subset = signer.setup_subset();

    Ok(subset)
}


pub async fn check(bytes: web::Bytes) -> Result<String, HttpResponse> {
    println!("check");

    let bytes = bytes.to_vec();
    let data = String::from_utf8_lossy(&bytes);
    let params: CheckParameter = get_body(&data)?;

    serde_json::to_string(&params).map_err(|e| HttpResponse::BadRequest().body(e.to_string()))
}
