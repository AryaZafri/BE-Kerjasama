use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{WebServiceResponse, query_ch};

#[derive(Debug, Deserialize)]
pub struct AddAgendaRequest {
    pub date: String,
    pub time_start: String,
    pub time_end: String,
    pub perundingan: String,
    pub jenis: String,
    pub pembahasan: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,
    username: String,
    email: String,
    role: String,
    exp: usize,
    iat: usize,
}

const JWT_SECRET: &str = "your-secret-key-change-in-production";

fn get_user_from_token(req: &HttpRequest) -> Result<Claims, String> {
    let cookie = req
        .cookie("auth_token")
        .ok_or("Token tidak ditemukan. Silakan login terlebih dahulu.")?;

    let token = cookie.value();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| format!("Token tidak valid: {}", e))?;

    Ok(token_data.claims)
}

pub async fn add_agenda(
    req: HttpRequest,
    data: web::Json<AddAgendaRequest>,
) -> impl Responder {
    let user = match get_user_from_token(&req) {
        Ok(u) => u,
        Err(e) => {
            return HttpResponse::Unauthorized().json(WebServiceResponse {
                status: "Error".into(),
                info: e,
            });
        }
    };

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
        "INSERT INTO kerjasama.agenda (date, time_start, time_end, perundingan, jenis, pembahasan, updated_by, timestamp) \
         VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', now())",
        data.date.replace("'", "''"),
        data.time_start.replace("'", "''"),
        data.time_end.replace("'", "''"),
        data.perundingan.replace("'", "''"),
        data.jenis.replace("'", "''"),
        data.pembahasan.replace("'", "''"),
        user.username.replace("'", "''")
    );

    println!("📝 Adding agenda:");
    println!("   Date: {}", data.date);
    println!("   Time: {} - {}", data.time_start, data.time_end);
    println!("   Perundingan: {}", data.perundingan);
    println!("   Jenis: {}", data.jenis);
    println!("   Updated by: {}", user.username);

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

pub async fn get_current_user(req: HttpRequest) -> impl Responder {
    match get_user_from_token(&req) {
        Ok(user) => HttpResponse::Ok().json(serde_json::json!({
            "status": "Ok",
            "data": {
                "id_user": user.sub,
                "username": user.username,
                "email": user.email,
                "role": user.role,
            }
        })),
        Err(e) => HttpResponse::Unauthorized().json(WebServiceResponse {
            status: "Error".into(),
            info: e,
        }),
    }
}