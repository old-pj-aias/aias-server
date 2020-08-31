pub mod handler;

use actix_web::{HttpResponse};
use serde::Deserialize;

pub fn get_body<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T, HttpResponse> {
    serde_json::from_slice(&data.as_bytes())
        .map_err(|e| HttpResponse::BadRequest().body(e.to_string()))
}