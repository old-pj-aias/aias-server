use actix_web::{HttpResponse, Responder, web};
use fair_blind_signature::CheckParameter;

use aias_core::signer::Signer;
use std::sync::{Arc, Mutex};

use crate::utils::{self, Keys};

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

    let conn = utils::db_connection();

    conn.execute("INSERT INTO sign_process (phone, m, subset)
                  VALUES ($1, $2, $3)",
                 &["000-000-0000", &body.to_string(), &subset]).unwrap();

    Ok(subset)
}


pub async fn check(body: web::Bytes, actix_data: web::Data<Arc<Mutex<Keys>>>) -> Result<String, HttpResponse> {
    println!("check");

    let id = 10;

    let signer_privkey = actix_data.lock().unwrap().signer_privkey.clone();
    let signer_pubkey = actix_data.lock().unwrap().signer_pubkey.clone();
    let judge_pubkey = actix_data.lock().unwrap().judge_pubkey.clone();


    let conn = utils::db_connection();
    let mut stmt = conn.prepare("SELECT phone, m, subset FROM sign_process WHERE id=?")
        .expect("failed to select");

    struct SubsetData {
        phone: String,
        m: String,
        subset: String
    }

    let SubsetData {phone, m, subset} = stmt.query_row(rusqlite::params![id], |row| {
        Ok(SubsetData {
            phone: row.get(0).unwrap(),
            m: row.get(1).unwrap(),
            subset: row.get(2).unwrap()
        })
    })
    .unwrap();

    let mut signer = Signer::new_from_params(signer_privkey, signer_pubkey, judge_pubkey, m, subset);

    let body = body.to_vec();
    let check_parameter = String::from_utf8_lossy(&body);

    if !signer.check(check_parameter.to_string()) {
        return Err(utils::bad_request("invalid"))
    }

    let signature = signer.sign();

    Ok(signature)
}
