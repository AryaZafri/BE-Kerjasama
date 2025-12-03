use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{WebServiceResponse, query_ch};

#[derive(Debug, Deserialize)]
pub struct DeleteAgendaRequest {
    pub date: String,
    pub time: String,  // Format: "09:30 - 11:00"
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

fn parse_time_range(time: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = time.split(" - ").collect();
    if parts.len() != 2 {
        return Err("Format waktu tidak valid. Gunakan format: HH:MM - HH:MM".to_string());
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

pub async fn delete_agenda(
    req: HttpRequest,
    data: web::Json<DeleteAgendaRequest>,
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

    if data.time.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Waktu harus diisi".into(),
        });
    }

    // Parse time range
    let (time_start, time_end) = match parse_time_range(&data.time) {
        Ok(times) => times,
        Err(e) => {
            return HttpResponse::BadRequest().json(WebServiceResponse {
                status: "Error".into(),
                info: e,
            });
        }
    };

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

    // Cek apakah agenda exists
    let check_query = format!(
        "SELECT * FROM db_kerjasama.agenda WHERE date = '{}' AND time_start = '{}' AND time_end = '{}' AND perundingan = '{}' AND jenis = '{}' AND pembahasan = '{}' LIMIT 1",
        data.date.replace("'", "''"),
        time_start.replace("'", "''"),
        time_end.replace("'", "''"),
        data.perundingan.replace("'", "''"),
        data.jenis.replace("'", "''"),
        data.pembahasan.replace("'", "''")
    );

    match query_ch(check_query).await {
        Ok(result) => {
            if let Some(s) = result.as_str() {
                if s.trim().is_empty() {
                    return HttpResponse::NotFound().json(WebServiceResponse {
                        status: "Error".into(),
                        info: "Agenda tidak ditemukan".into(),
                    });
                }
            } else {
                if let Some(arr) = result.as_array() {
                    if arr.is_empty() {
                        return HttpResponse::NotFound().json(WebServiceResponse {
                            status: "Error".into(),
                            info: "Agenda tidak ditemukan".into(),
                        });
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to check agenda: {}", e);
            return HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: format!("Gagal memeriksa agenda: {}", e),
            });
        }
    }

    // ========== DELETE QUERY ==========
    let query = format!(
        "ALTER TABLE db_kerjasama.agenda DELETE WHERE date = '{}' AND time_start = '{}' AND time_end = '{}' AND perundingan = '{}' AND jenis = '{}' AND pembahasan = '{}'",
        data.date.replace("'", "''"),
        time_start.replace("'", "''"),
        time_end.replace("'", "''"),
        data.perundingan.replace("'", "''"),
        data.jenis.replace("'", "''"),
        data.pembahasan.replace("'", "''")
    );

    println!("🗑️  Deleting agenda:");
    println!("   Date: {}", data.date);
    println!("   Time: {}", data.time);
    println!("   Perundingan: {}", data.perundingan);
    println!("   Jenis: {}", data.jenis);
    println!("   Pembahasan: {}", data.pembahasan);
    println!("   Deleted by: {}", user.username);

    match query_ch(query).await {
        Ok(_) => {
            println!("✅ Agenda deleted successfully");
            HttpResponse::Ok().json(serde_json::json!({
                "status": "Ok",
                "info": "Agenda berhasil dihapus",
                "data": {
                    "date": data.date,
                    "time": data.time,
                    "perundingan": data.perundingan,
                    "jenis": data.jenis,
                    "pembahasan": data.pembahasan,
                    "deleted_by": user.username
                }
            }))
        }
        Err(e) => {
            eprintln!("❌ Failed to delete agenda: {}", e);
            HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: format!("Gagal menghapus agenda: {}", e),
            })
        }
    }
}