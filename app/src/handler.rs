use std::sync::{Arc, Mutex};
use std::env;

use actix_web::{HttpResponse, Responder, web, HttpRequest};
use actix_session::{Session, CookieSession};

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use serde::{Deserialize, Serialize};
use fair_blind_signature::CheckParameter;
use aias_core::signer::{Signer, ReadyParams};

use crate::utils::{self, Keys};



pub async fn hello() -> impl Responder {
    println!("hello");
    
    HttpResponse::Ok().body("Hello world")
}

pub async fn send_sms(body: web::Bytes, session: Session) -> Result<String, HttpResponse> {
    println!("send_sms");

    let is_debugging = env::var("DEBUGGING").expect("Find DEBUGGING environment variable");

    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .collect();
    
    #[derive(Deserialize, Serialize)]
    struct PhoneReq {
        phone_number: String,
    }

    let phone_number_str = String::from_utf8_lossy(&body).to_string();
    let phone_number : PhoneReq = serde_json::from_str(&phone_number_str).map_err(utils::internal_server_error)?;

    session.set("phone-number", &phone_number_str)?;

    let mut rng = thread_rng();
    let secret: u32 = rng.gen_range(100000, 999999);
    let secret = secret.to_string();

    let conn = utils::db_connection();

    conn.execute("INSERT INTO users (phone, token)
                  VALUES ($1, $2)",
                 &[phone_number_str.clone(), token]).unwrap();

    let id = conn.last_insert_rowid();

    conn.execute("INSERT INTO sms_codes (id, secret_code)
                VALUES ($1, $2)",
                &[id.clone().to_string(), secret.clone()]).unwrap();

    if is_debugging == "true" {
        env::set_var("TEST_SECRET_CODE", secret.clone());
        println!("secret: {}", secret);
    } 
    else {
        utils::send_sms(phone_number.phone_number, secret);
    }

    session.set("id", id)?;


    Ok("{}".to_string())
}

pub async fn verify_code(body: web::Bytes, session: Session) -> Result<String, HttpResponse> {
    println!("verify_code");

    if let Some(count) = session.get::<i32>("counter")? {
        if count >= 5 {
            return Err(utils::bad_request("too many login failures"));
        }
        session.set("counter", count + 1)?;
    } else {
        session.set("counter", 1)?;
    }

    #[derive(Deserialize, Serialize)]
    struct CodeReq {
        code: u32,
    }

    let code_str = String::from_utf8_lossy(&body).to_string();
    let code : CodeReq = serde_json::from_str(&code_str).map_err(utils::internal_server_error)?;

    let id = session.get::<u32>("id").unwrap().unwrap();

    // access session data
    let conn = utils::db_connection();
    let mut stmt = conn.prepare("SELECT secret_code FROM sms_codes WHERE id=?")
        .expect("failed to select");

    struct CorrectCode {
        correct_code: u32
    }

    let CorrectCode {correct_code} = stmt.query_row(rusqlite::params![id], |row| {
        Ok(CorrectCode {
            correct_code: row.get(0).unwrap(),
        })
    })
    .unwrap();


    if code.code != correct_code {
        return Err(utils::bad_request("invalid code"));
    }

    #[derive(Deserialize, Serialize)]
    struct TokenResp {
        token: String
    }

    let mut stmt = conn.prepare("SELECT token FROM users WHERE id=?")
        .expect("failed to select");


    let TokenResp {token} = stmt.query_row(rusqlite::params![id], |row| {
        Ok(TokenResp {
            token: row.get(0).unwrap(),
        })
    })
    .unwrap();

    let cloned_token = token.clone();
    let resp = TokenResp { token: cloned_token };
    let resp = serde_json::to_string(&resp).unwrap();

    let is_debugging = env::var("DEBUGGING").expect("Find DEBUGGIN environment variable");

    if is_debugging == "true" {
        env::set_var("TEST_TOKEN", token.clone());
        println!("token: {}", token);
    }

    Ok(resp)
}

pub async fn auth(body: web::Bytes, session: Session) -> Result<String, HttpResponse> {
    let conn = utils::db_connection();

     #[derive(Deserialize, Serialize)]
    struct TokenReq {
        token: String,
    }

    let token_str = String::from_utf8_lossy(&body).to_string();
    let token : TokenReq = serde_json::from_str(&token_str).map_err(utils::internal_server_error)?;
    
    #[derive(Deserialize, Serialize)]
    struct IdResp {
        id: u32,
    }

    let mut stmt = conn.prepare("SELECT id FROM users WHERE token=?")
        .expect("failed to select");


    let IdResp { id } = stmt.query_row(rusqlite::params![token.token], |row| {
        Ok(IdResp {
            id: row.get(0).unwrap(),
        })
    })
    .unwrap();

    session.set("id", id)?;

    let cloned_id = id.clone();
    let resp = IdResp { id: cloned_id };
    let resp = serde_json::to_string(&resp).unwrap();

    let is_debugging = env::var("DEBUGGING").expect("Find DEBUGGIN environment variable");

    if is_debugging == "true" {
        env::set_var("TEST_ID", id.to_string());
        println!("id: {}", id);
    }

    Ok(resp)
}


pub async fn ready(body: web::Bytes, session: Session, actix_data: web::Data<Arc<Mutex<Keys>>>) -> Result<String, HttpResponse> {
    println!("ready");


    let signer_privkey = actix_data.lock().unwrap().signer_privkey.clone();
    let signer_pubkey = actix_data.lock().unwrap().signer_pubkey.clone();

    let ready_params_str = String::from_utf8_lossy(&body).to_string();

    let digest_and_ej = serde_json::from_str(&ready_params_str).expect("failed to parse json");
    let ReadyParams { judge_pubkey, blinded_digest } = digest_and_ej;

    let id = session.get::<u32>("id").unwrap().unwrap();

    let mut signer = Signer::new_with_blinded_digest(signer_privkey, signer_pubkey, ready_params_str.clone(), id);

    let subset_str: String = signer.setup_subset();
    let blinded_digest_str = serde_json::to_string(&blinded_digest).unwrap().to_string();

    let conn = utils::db_connection();
    conn.execute("INSERT INTO sign_process (id, blinded_digest, subset, judge_pubkey)
                  VALUES ($1, $2, $3, $4)",
                 &[id.to_string(), blinded_digest_str, subset_str.clone(), judge_pubkey]).unwrap();

    Ok(subset_str)
}


pub async fn sign(body: web::Bytes, session: Session, actix_data: web::Data<Arc<Mutex<Keys>>>) -> Result<String, HttpResponse> {
    println!("sign");

    let signer_privkey = actix_data.lock().unwrap().signer_privkey.clone();
    let signer_pubkey = actix_data.lock().unwrap().signer_pubkey.clone();

    let id = session.get::<u32>("id").unwrap().unwrap();

    let conn = utils::db_connection();
    let mut stmt = conn.prepare("SELECT blinded_digest, subset, judge_pubkey FROM sign_process WHERE id=?")
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

    match signer.check(check_parameter.to_string()) {
        Ok(_) => {},
        Err(e) => {
            return Err(utils::bad_request("invalid"));
        }
    }

    conn.execute("DELETE FROM sign_process WHERE id=?", &[id]).unwrap();

    let signature = signer.sign();

    Ok(signature)
}
