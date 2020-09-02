use super::*;
use actix_web::{test, web, App, HttpMessage};
use aias_core::{judge, sender, signer, verifyer};

use serde::{Deserialize, Serialize};

use std::fs;
use std::sync::{Arc, Mutex};
use std::iter::Iterator;

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
            .wrap(
                CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                    .secure(false)
            )
            .data(data.clone())
            .route("/verify_code", web::get().to(handler::verify_code))
            .route("/ready", web::post().to(handler::ready))
            .route("/sign", web::post().to(handler::sign))
            .route("/hello", web::get().to(handler::hello))
    )
    .await;

    let req = test::TestRequest::get().uri("/verify_code").to_request();
    let resp = test::call_service(&mut app, req).await;
    let resp = resp.response();

    let cookies =
        resp
        .cookies();

    let l: usize = cookies.map(|c| println!("cookies: {:?}", c)).fold(0, |l,_| l + 1);
    println!("length: {}", l);
    
    let cookie = 
        resp
        .cookies()
        .find(|c| c.name() == "actix-session")
        .expect("failed to get id from response's session");

    #[derive(Deserialize, Serialize)]
    struct IdResp {
        id: u32,
    }

    /*
    // I wanted to get response's body, but I couldn't find the proper way
    let body = response.body();
    let IdResp { id } = serde_json::from_str(&body).unwrap();
    */
    let id = 10;

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
