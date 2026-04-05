use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::{WebServiceResponse, query_ch};

#[derive(Debug, Deserialize)]
pub struct AddAgendaRequest {
    pub date: String,
    pub time_start: String,
    pub time_end: String,
    pub perundingan: String,
    pub jenis: String,
    pub pembahasan: String,
    pub updated_by: Option<String>,
}

pub async fn add_agenda(
    data: web::Json<AddAgendaRequest>,
) -> impl Responder {

    // ========== VALIDASI ==========
    if data.date.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Tanggal harus diisi".into(),
        });
    }

    if !data.date.contains('-') || data.date.len() != 10 {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Format tanggal harus YYYY-MM-DD".into(),
        });
    }

    if data.time_start.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Waktu mulai harus diisi".into(),
        });
    }

    if !data.time_start.contains(':') || data.time_start.len() != 5 {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Format waktu mulai harus HH:MM (contoh: 10:00)".into(),
        });
    }

    if data.time_end.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Waktu selesai harus diisi".into(),
        });
    }

    if !data.time_end.contains(':') || data.time_end.len() != 5 {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Format waktu selesai harus HH:MM (contoh: 12:00)".into(),
        });
    }

    if data.time_end <= data.time_start {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Waktu selesai harus lebih besar dari waktu mulai".into(),
        });
    }

    if data.perundingan.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Perundingan harus diisi".into(),
        });
    }

    if data.jenis.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Jenis harus diisi".into(),
        });
    }

    if data.pembahasan.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Pembahasan harus diisi".into(),
        });
    }

    // ========== INSERT QUERY ==========
    let query = format!(
        "INSERT INTO db_kerjasama.agenda (date, time_start, time_end, perundingan, jenis, pembahasan, updated_by, timestamp) \
         VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', now())",
        data.date.replace("'", "''"),
        data.time_start.replace("'", "''"),
        data.time_end.replace("'", "''"),
        data.perundingan.replace("'", "''"),
        data.jenis.replace("'", "''"),
        data.pembahasan.replace("'", "''"),
        data.updated_by.clone().unwrap_or_default().replace("'", "''")
    );

    println!("📝 Adding agenda:");
    println!("   Date: {}", data.date);
    println!("   Time: {} - {}", data.time_start, data.time_end);
    println!("   Perundingan: {}", data.perundingan);
    println!("   Jenis: {}", data.jenis);

    match query_ch(query).await {
        Ok(_) => {
            println!("✅ Agenda added successfully");
            HttpResponse::Ok().json(serde_json::json!({
                "status": "Ok",
                "info": "Agenda berhasil ditambahkan",
                "data": {
                    "date": data.date,
                    "time_start": data.time_start,
                    "time_end": data.time_end,
                    "perundingan": data.perundingan,
                    "jenis": data.jenis
                }
            }))
        }
        Err(e) => {
            eprintln!("❌ Failed to add agenda: {}", e);
            HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: format!("Gagal menambahkan agenda: {}", e),
            })
        }
    }
}

