use actix_web::{HttpResponse, Responder, web};
use fair_blind_signature::CheckParameter;

use aias_core::signer::{Signer, ReadyParams};
use std::sync::{Arc, Mutex};

use crate::utils::{self, Keys};

pub async fn hello() -> impl Responder {
    println!("hello");
    
    HttpResponse::Ok().body("Hello world")
}


pub async fn ready(body: web::Bytes, actix_data: web::Data<Arc<Mutex<Keys>>>) -> Result<String, HttpResponse> {
    println!("ready");

    let id = 10;

    let signer_privkey = actix_data.lock().unwrap().signer_privkey.clone();
    let signer_pubkey = actix_data.lock().unwrap().signer_pubkey.clone();

    let ready_params_str = String::from_utf8_lossy(&body).to_string();

    let digest_and_ej = serde_json::from_str(&ready_params_str).expect("failed to parse json");
    let ReadyParams { judge_pubkey, blinded_digest } = digest_and_ej;

    let mut signer = Signer::new_with_blinded_digest(signer_privkey, signer_pubkey, ready_params_str.clone(), id);

    let subset_str: String = signer.setup_subset();
    let blinded_digest_str = serde_json::to_string(&blinded_digest).unwrap();

    let conn = utils::db_connection();

    conn.execute("INSERT INTO sign_process (phone, blinded_digest, subset, session_id, judge_pubkey)
                  VALUES ($1, $2, $3, $4, $5)",
                 &[id.to_string(), blinded_digest_str, subset_str.clone(), id.to_string(), judge_pubkey]).unwrap();

    Ok(subset_str)
}


pub async fn sign(body: web::Bytes, actix_data: web::Data<Arc<Mutex<Keys>>>) -> Result<String, HttpResponse> {
    println!("sign");

    let id = 10;

    let signer_privkey = actix_data.lock().unwrap().signer_privkey.clone();
    let signer_pubkey = actix_data.lock().unwrap().signer_pubkey.clone();

    let conn = utils::db_connection();
    let mut stmt = conn.prepare("SELECT blinded_digest, subset, judge_pubkey FROM sign_process WHERE session_id=?")
        .expect("failed to select");

    struct SignData {
        blinded_digest: String,
        subset: String,
        judge_pubkey: String
    }

    let SignData {blinded_digest, subset, judge_pubkey} = stmt.query_row(rusqlite::params![id], |row| {
        Ok(SignData {
            blinded_digest: row.get(0).unwrap(),
            subset: row.get(1).unwrap(),
            judge_pubkey: row.get(2).unwrap()
        })
    })
    .unwrap();

    let mut signer = Signer::new_from_params(signer_privkey, signer_pubkey, judge_pubkey, id, blinded_digest, subset);

    let body = body.to_vec();
    let check_parameter = String::from_utf8_lossy(&body);

    if !signer.check(check_parameter.to_string()) {
        return Err(utils::bad_request("invalid"));
    }

    let signature = signer.sign();

    Ok(signature)
}
