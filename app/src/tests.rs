use super::*;
use actix_web::{test, web, App};
use aias_core::{judge, sender, verifyer};

use std::fs;
use std::sync::{Arc, Mutex};

use futures::stream::poll_fn;
use rusqlite::params;


#[actix_rt::test]
async fn test() {
    if let Err(e) = fs::remove_file("db.sqlite3") {
        eprintln!("an error occured on removing db data: {}", e);
    }

    let conn = utils::db_connection();

    if let Err(e) = conn.execute(
        "CREATE TABLE sign_process (
                  id              INTEGER PRIMARY KEY,
                  phone           TEXT NOT NULL,
                  m               TEXT NOT NULL,
                  subset          TEXT NOT NULL
                  )",
        params![],
    ) {
        eprintln!("an error occured on creating table: {}", e);
    }

    let signer_pubkey = fs::read_to_string("keys/signer_pubkey.pem").unwrap();
    let signer_privkey = fs::read_to_string("keys/signer_privkey.pem").unwrap();
    let judge_pubkey = fs::read_to_string("keys/judge_pubkey.pem").unwrap();

    sender::new(signer_pubkey.clone(), judge_pubkey.clone());
    let blinded_digest = sender::blind("hoge".to_string());

    let data = Arc::new(Mutex::new(Keys {
        signer_pubkey: signer_pubkey.clone(),
        signer_privkey: signer_privkey.clone(),
    }));

    let mut app = test::init_service(
        App::new()
        .data(data)
        .route("/ready", web::post().to(handler::ready))
        .route("/sign", web::post().to(handler::sign))).await;

    let req = test::TestRequest::post().uri("/ready").set_payload(blinded_digest).to_request();
    let resp = test::call_service(&mut app, req).await;

    let bytes = test::read_body(resp).await;
    let subset = String::from_utf8(bytes.to_vec()).unwrap();

    sender::set_subset(subset);
    let params = sender::generate_check_parameters();

    let req = test::TestRequest::post().uri("/sign").set_payload(params).to_request();
    let resp = test::call_service(&mut app, req).await;

    let bytes = test::read_body(resp).await;
    let blind_signature = String::from_utf8(bytes.to_vec()).unwrap();

    let signature = sender::unblind(blind_signature);

    let result = verifyer::verify(signature, "hoge".to_string(), signer_pubkey, judge_pubkey);

    assert!(result);
}
