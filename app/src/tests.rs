use super::*;
use actix_web::{test, web, App, HttpMessage};
use aias_core::{judge, sender, signer, verifyer};

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

    utils::create_table_sign_process().unwrap_or_else(|e| {
        eprintln!("error creating table: {}", e);
    });

    let signer_pubkey = fs::read_to_string("keys/signer_pubkey.pem").unwrap();
    let signer_privkey = fs::read_to_string("keys/signer_privkey.pem").unwrap();
    let judge_pubkey = fs::read_to_string("keys/judge_pubkey.pem").unwrap();

    let data = Arc::new(Mutex::new(Keys {
        signer_pubkey: signer_pubkey.clone(),
        signer_privkey: signer_privkey.clone(),
    }));

    let mut app = test::init_service(
        App::new()
        .data(data)
        .route("/send_id", web::get().to(handler::send_id))
        .route("/ready", web::post().to(handler::ready))
        .route("/sign", web::post().to(handler::sign)))
        .await;

    let req = test::TestRequest::get().uri("/send_id").to_request();
    let resp = test::call_service(&mut app, req).await;

    let cookie =
        resp
        .response()
        .cookies()
        .find(|c| c.name() == "id")
        .expect("failed to get id from response's session");

    let id: u32 =
        cookie
        .value()
        .parse()
        .expect("failed to parse session");

    sender::new(signer_pubkey.clone(), judge_pubkey.clone(), id);
    let blinded_digest_str = sender::blind("hoge".to_string());
    let blinded_digest = serde_json::from_str(&blinded_digest_str).expect("failed to parse json");

    let ready_params = signer::ReadyParams {
        judge_pubkey: judge_pubkey.clone(),
        blinded_digest
    };

    let ready_params_str = serde_json::to_string(&ready_params).expect("failed to convet into json");

    let req =
        test::TestRequest::post()
        .uri("/ready")
        .set_payload(ready_params_str.clone())
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&mut app, req).await;

    let bytes = test::read_body(resp).await;
    let subset = String::from_utf8(bytes.to_vec()).unwrap();

    sender::set_subset(subset);
    let params = sender::generate_check_parameters();

    let req =
        test::TestRequest::post()
        .uri("/sign")
        .set_payload(params.clone())
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        let signer_pubkey = fs::read_to_string("keys/signer_pubkey.pem").unwrap();
        let signer_privkey = fs::read_to_string("keys/signer_privkey.pem").unwrap();
        let mut signer = aias_core::signer::Signer::new_with_blinded_digest(signer_privkey, signer_pubkey, ready_params_str, id);
        assert!(signer.check(params));
    }

    let bytes = test::read_body(resp).await;
    let blind_signature = String::from_utf8(bytes.to_vec()).unwrap();

    println!("response: {}", blind_signature);
    let signature = sender::unblind(blind_signature);

    let result = verifyer::verify(signature, "hoge".to_string(), signer_pubkey, judge_pubkey);

    assert!(result);
}
