use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;
use std::fs;

use crate::{WebServiceResponse, query_ch};

#[derive(Debug, Deserialize)]
pub struct DeleteDocumentRequest {
    pub perundingan: String,
    pub negara_mitra: String,
    pub jenis: String,
    pub last_update: String,
    pub pembahasan: String,
    pub document: String,
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

pub async fn delete_document(
    req: HttpRequest,
    data: web::Json<DeleteDocumentRequest>,
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
    if data.perundingan.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Perundingan harus diisi".into(),
        });
    }

    if data.negara_mitra.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Negara mitra harus diisi".into(),
        });
    }

    if data.jenis.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Jenis harus diisi".into(),
        });
    }

    if data.last_update.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Last update harus diisi".into(),
        });
    }

    if data.pembahasan.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Pembahasan harus diisi".into(),
        });
    }

    if data.document.is_empty() {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Document path harus diisi".into(),
        });
    }

    // Cek apakah document exists
    let check_query = format!(
        "SELECT * FROM db_kerjasama.document WHERE perundingan = '{}' AND negara_mitra = '{}' AND jenis = '{}' AND last_update = '{}' AND pembahasan = '{}' AND document = '{}' LIMIT 1",
        data.perundingan.replace("'", "''"),
        data.negara_mitra.replace("'", "''"),
        data.jenis.replace("'", "''"),
        data.last_update.replace("'", "''"),
        data.pembahasan.replace("'", "''"),
        data.document.replace("'", "''")
    );

    match query_ch(check_query).await {
        Ok(result) => {
            if let Some(s) = result.as_str() {
                if s.trim().is_empty() {
                    return HttpResponse::NotFound().json(WebServiceResponse {
                        status: "Error".into(),
                        info: "Document tidak ditemukan".into(),
                    });
                }
            } else {
                if let Some(arr) = result.as_array() {
                    if arr.is_empty() {
                        return HttpResponse::NotFound().json(WebServiceResponse {
                            status: "Error".into(),
                            info: "Document tidak ditemukan".into(),
                        });
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to check document: {}", e);
            return HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: format!("Gagal memeriksa document: {}", e),
            });
        }
    }

    // ========== DELETE QUERY ==========
    let query = format!(
        "ALTER TABLE db_kerjasama.document DELETE WHERE perundingan = '{}' AND negara_mitra = '{}' AND jenis = '{}' AND last_update = '{}' AND pembahasan = '{}' AND document = '{}'",
        data.perundingan.replace("'", "''"),
        data.negara_mitra.replace("'", "''"),
        data.jenis.replace("'", "''"),
        data.last_update.replace("'", "''"),
        data.pembahasan.replace("'", "''"),
        data.document.replace("'", "''")
    );

    println!("🗑️  Deleting document:");
    println!("   Perundingan: {}", data.perundingan);
    println!("   Negara Mitra: {}", data.negara_mitra);
    println!("   Jenis: {}", data.jenis);
    println!("   Last Update: {}", data.last_update);
    println!("   Pembahasan: {}", data.pembahasan);
    println!("   Document Path: {}", data.document);
    println!("   Deleted by: {}", user.username);

    match query_ch(query).await {
        Ok(_) => {
            // Hapus file fisik jika ada
            if !data.document.is_empty() {
                if let Err(e) = fs::remove_file(&data.document) {
                    eprintln!("⚠️  Warning: Failed to delete physical file {}: {}", data.document, e);
                } else {
                    println!("✅ Physical file deleted: {}", data.document);
                }
            }

            println!("✅ Document deleted successfully");
            HttpResponse::Ok().json(json!({
                "status": "Ok",
                "info": "Document berhasil dihapus",
                "data": {
                    "perundingan": data.perundingan,
                    "negara_mitra": data.negara_mitra,
                    "jenis": data.jenis,
                    "last_update": data.last_update,
                    "pembahasan": data.pembahasan,
                    "document": data.document,
                    "deleted_by": user.username
                }
            }))
        }
        Err(e) => {
            eprintln!("❌ Failed to delete document: {}", e);
            HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: format!("Gagal menghapus document: {}", e),
            })
        }
    }
}