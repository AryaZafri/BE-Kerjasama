use actix_web::web;
use reqwest::{redirect::Policy, Client, Result};

use crate::{add_agenda, login, register, verify_account};

pub fn get_client() -> Result<Client> {
    reqwest::ClientBuilder::new()
        .cookie_store(true)
        .danger_accept_invalid_certs(true)
        .redirect(Policy::limited(20))
        .build()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg 
            // <-----------------------------------------AUTHORIZATION----------------------------------------->
            // .route("/login", web::post().to(login))
            
            // REGISTER & VERIFICATION
            .route("/register", web::post().to(register))
            .route("/verify/{token}", web::get().to(verify_account))
            .route("/login", web::post().to(login))

            // DASHBOARD
            .route("/add_agenda", web::post().to(add_agenda))
            ;
}