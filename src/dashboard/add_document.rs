use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder};
use futures_util::stream::StreamExt as _;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;
use chrono::Local;

use crate::{WebServiceResponse, query_ch};

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

pub async fn add_document(mut payload: Multipart, req: HttpRequest) -> impl Responder {
    let user = match get_user_from_token(&req) {
        Ok(u) => u,
        Err(e) => {
            return HttpResponse::Unauthorized().json(WebServiceResponse {
                status: "Error".into(),
                info: e,
            });
        }
    };

    let base_path = "/home/app/fe/document";
    
    if let Err(e) = fs::create_dir_all(base_path) {
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to create directory: {}", e)
        }));
    }

    let mut file_path = String::new();
    let mut original_filename = String::new();
    
    let mut perundingan = String::new();
    let mut negara_mitra = String::new();
    let mut jenis = String::new();
    let mut last_update = String::new();
    let mut pembahasan = String::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("").to_string();

        match field_name.as_str() {
            "perundingan" | "negara_mitra" | "jenis" | "last_update" | "pembahasan" => {
                let mut field_data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(d) => d,
                        Err(e) => {
                            return HttpResponse::BadRequest().json(json!({
                                "status": "error",
                                "message": format!("Failed to read field {}: {}", field_name, e)
                            }));
                        }
                    };
                    field_data.extend_from_slice(&data);
                }

                let value = String::from_utf8_lossy(&field_data).to_string();
                
                match field_name.as_str() {
                    "perundingan" => perundingan = value,
                    "negara_mitra" => negara_mitra = value,
                    "jenis" => jenis = value,
                    "last_update" => last_update = value,
                    "pembahasan" => pembahasan = value,
                    _ => {}
                }
            }
            "file" => {
                if let Some(filename) = content_disposition.get_filename() {
                    original_filename = filename.to_string();
                    
                    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                    let extension = Path::new(filename)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("");
                    
                    let new_filename = if !extension.is_empty() {
                        format!("{}_{}.{}", 
                            Path::new(filename).file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("file"),
                            timestamp,
                            extension
                        )
                    } else {
                        format!("{}_{}", filename, timestamp)
                    };
                    
                    file_path = format!("{}/{}", base_path, new_filename);
                    
                    let mut f = match fs::File::create(&file_path) {
                        Ok(file) => file,
                        Err(e) => {
                            return HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": format!("Failed to create file: {}", e)
                            }));
                        }
                    };

                    while let Some(chunk) = field.next().await {
                        let data = match chunk {
                            Ok(d) => d,
                            Err(e) => {
                                let _ = fs::remove_file(&file_path);
                                return HttpResponse::InternalServerError().json(json!({
                                    "status": "error",
                                    "message": format!("Failed to read chunk: {}", e)
                                }));
                            }
                        };
                        
                        if let Err(e) = f.write_all(&data) {
                            let _ = fs::remove_file(&file_path);
                            return HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": format!("Failed to write file: {}", e)
                            }));
                        }
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }

    if file_path.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "No file uploaded"
        }));
    }

    if perundingan.is_empty() || negara_mitra.is_empty() 
        || jenis.is_empty() || last_update.is_empty() 
        || pembahasan.is_empty() {
        let _ = fs::remove_file(&file_path);
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Missing required fields (perundingan, negara_mitra, jenis, last_update, pembahasan)"
        }));
    }

    let query = format!(
        "INSERT INTO db_kerjasama.document (perundingan, negara_mitra, jenis, last_update, pembahasan, document, uploded_by) 
         VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}')",
        perundingan.replace("'", "''"),
        negara_mitra.replace("'", "''"),
        jenis.replace("'", "''"),
        last_update.replace("'", "''"),
        pembahasan.replace("'", "''"),
        file_path.replace("'", "''"),
        user.username.replace("'", "''")
    );

    match query_ch(query).await {
        Ok(_) => {
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Document uploaded successfully",
                "data": {
                    "original_filename": original_filename,
                    "file_path": file_path,
                    "perundingan": perundingan,
                    "negara_mitra": negara_mitra,
                    "uploaded_by": user.username
                }
            }))
        }
        Err(e) => {
            let _ = fs::remove_file(&file_path);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to insert to database: {}", e)
            }))
        }
    }
}