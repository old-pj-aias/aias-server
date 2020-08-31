use super::*;
use actix_web::{test, web, App};
use aias_core::{judge, sender};

use std::fs;
use std::sync::{Arc, Mutex};

use futures::stream::poll_fn;
use rusqlite::params;


#[actix_rt::test]
async fn test() {
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

    let signer_pubkey = fs::read_to_string("keys/signer_pubkey.pem").unwrap();
    let signer_privkey = fs::read_to_string("keys/signer_privkey.pem").unwrap();
    let judge_pubkey = fs::read_to_string("keys/judge_pubkey.pem").unwrap();

    sender::new(signer_pubkey.clone(), judge_pubkey.clone());
    let blinded_digest = sender::blind("hoge".to_string());

    let data = Arc::new(Mutex::new(Keys {
        signer_pubkey: signer_pubkey,
        signer_privkey: signer_privkey,
        judge_pubkey: judge_pubkey
    }));

    let mut app = test::init_service(
        App::new()
        .data(data)
        .route("/ready", web::post().to(handler::ready))).await;

    let req = test::TestRequest::post().uri("/ready").set_payload(blinded_digest).to_request();
    let resp = test::call_service(&mut app, req).await;

    let bytes = test::read_body(resp).await;
    let body = String::from_utf8(bytes.to_vec()).unwrap();


    println!("{:?}", body);
    // assert!(resp.status().is_client_error());
}