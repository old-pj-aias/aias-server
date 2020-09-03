use super::*;
use actix_web::{test, web, App, HttpMessage};
use aias_core::{judge, sender, signer, verifyer};

use serde::{Deserialize, Serialize};

use std::fs;
use std::sync::{Arc, Mutex};
use std::iter::Iterator;

use futures::stream::poll_fn;
use rusqlite::params;

use std::env;

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
            .route("/send_sms", web::post().to(handler::send_sms))
            .route("/verify_code", web::post().to(handler::verify_code))
            .route("/ready", web::post().to(handler::ready))
            .route("/sign", web::post().to(handler::sign))
            .route("/hello", web::get().to(handler::hello))
    )
    .await;

    #[derive(Deserialize, Serialize)]
    struct PhoneReq {
        phone_number: String,
    }

    let phone_num = env::var("TO").expect("Find TO environment variable");
    let phone_req = PhoneReq { phone_number: phone_num };
    let phone_req = serde_json::to_string(&phone_req).unwrap();

    let req = test::TestRequest::post().uri("/send_sms").set_payload(phone_req).to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    
    let resp = resp.response();

    let cookie = 
        resp
        .cookies()
        .find(|c| c.name() == "actix-session")
        .expect("failed to get id from response's session");

    let code = env::var("TEST_SECRET_CODE").expect("Find SECRET environment variable");
    let code = code.parse().unwrap();

    #[derive(Deserialize, Serialize)]
    struct CodeReq {
        code: u32,
    }

    let code_req = CodeReq {
        code: code
    };

    let code_req = serde_json::to_string(&code_req).unwrap();

    let req = test::TestRequest::post().uri("/verify_code").cookie(cookie.clone()).set_payload(code_req).to_request();
    let resp = test::call_service(&mut app, req).await;

    #[derive(Deserialize, Serialize)]
    struct TokenAndId {
        id: u32,
        token: String
    }

    let bytes = test::read_body(resp).await;
    let token_resp = String::from_utf8(bytes.to_vec()).unwrap();

    let TokenAndId { id, token } = serde_json::from_str(&token_resp).unwrap();

    let test_token = env::var("TEST_TOKEN").expect("Find TEST_ID environment variable");

    assert_eq!(token, test_token);

    let test_id = env::var("TEST_ID").expect("Find TEST_ID environment variable");
    let test_id : u32 = test_id.parse().unwrap();

    assert_eq!(id, test_id);

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

// #[test]
// fn test_call(){
//     let to = env::var("TO").expect("Find TO environment variable");
//     let msg = "hello".to_string();

//     utils::send_sms(to.to_string(), msg);
// }
