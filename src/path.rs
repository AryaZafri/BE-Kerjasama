use actix_web::web;
use reqwest::{redirect::Policy, Client, Result};

use crate::{add_agenda, add_document, delete_agenda, delete_document, get_agenda, get_document, kerjasama_proses_berlaku, komoditas_utama, list_direktorat, list_kerjasama, list_perundingan, list_year, login, register, top_three_category, total_kerjasama, trade_balance, tren_ekspor_impor, tren_komoditas_utama, utilitas_kerjasama, verify_account};

pub fn get_client() -> Result<Client> {
    reqwest::ClientBuilder::new()
        .cookie_store(true)
        .danger_accept_invalid_certs(true)
        .redirect(Policy::limited(20))
        .build()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg 
            // REGISTER & VERIFICATION
            .route("/register", web::post().to(register))
            .route("/verify/{token}", web::get().to(verify_account))
            .route("/login", web::post().to(login))

            // DASHBOARD
            .route("/add_agenda", web::post().to(add_agenda))
            .route("/get_agenda", web::post().to(get_agenda))
            .route("/delete_agenda", web::post().to(delete_agenda))
            .route("/add_document", web::post().to(add_document))
            .route("/get_document", web::post().to(get_document))
            .route("/delete_document", web::post().to(delete_document))
            .route("/total_kerjasama", web::post().to(total_kerjasama))
            .route("/kerjasama_proses_berlaku", web::post().to(kerjasama_proses_berlaku))
            .route("/list_kerjasama", web::post().to(list_kerjasama))
            .route("/top_three_category", web::post().to(top_three_category))
            .route("/utilitas_kerjasama", web::post().to(utilitas_kerjasama))
            .route("/list_perundingan", web::get().to(list_perundingan))
            .route("/list_direktorat", web::get().to(list_direktorat))

            // ANALYTICS
            .route("/tren_ekspor_impor", web::post().to(tren_ekspor_impor))
            .route("/trade_balance", web::post().to(trade_balance))
            .route("/komoditas_utama", web::post().to(komoditas_utama))
            .route("/tren_komoditas_utama", web::post().to(tren_komoditas_utama))
            .route("/list_year", web::get().to(list_year))
            ;
}