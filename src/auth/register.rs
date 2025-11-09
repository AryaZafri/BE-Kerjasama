use actix_web::{web, HttpResponse, Responder};

use crate::{UserAccessRegister, WebServiceResponse, query_ch};

pub async fn register(data: web::Json<UserAccessRegister>) -> impl Responder {
    if data.password != data.confirm_pass {
        return HttpResponse::BadRequest().json(WebServiceResponse {
            status: "Error".into(),
            info: "Password dan konfirmasi password tidak cocok".into(),
        });
    }

    let bcrypt_password = match bcrypt::hash(&data.password, bcrypt::DEFAULT_COST) {
        Ok(hashed) => hashed,
        Err(_) => {
            return HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: "Gagal melakukan hashing password".into(),
            });
        }
    };

    let email = data.email.clone().unwrap_or_default();
    let check_email_query = format!(
        "SELECT count() as cnt FROM kerjasama.user WHERE email = '{}' FORMAT JSON",
        email.replace("'", "''")
    );

    match query_ch(check_email_query).await {
        Ok(result) => {
            if let Some(data_arr) = result.get("data").and_then(|d| d.as_array()) {
                if let Some(first_row) = data_arr.first() {
                    if let Some(count) = first_row.get("cnt").and_then(|v| v.as_i64()) {
                        if count > 0 {
                            return HttpResponse::BadRequest().json(WebServiceResponse {
                                status: "Error".into(),
                                info: "Email sudah terdaftar".into(),
                            });
                        }
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: format!("Gagal memeriksa email: {}", e),
            });
        }
    }

    let username = data.username.clone().unwrap_or_default();
    let check_username_query = format!(
        "SELECT count() as cnt FROM kerjasama.user WHERE username = '{}' FORMAT JSON",
        username.replace("'", "''")
    );

    match query_ch(check_username_query).await {
        Ok(result) => {
            if let Some(data_arr) = result.get("data").and_then(|d| d.as_array()) {
                if let Some(first_row) = data_arr.first() {
                    if let Some(count) = first_row.get("cnt").and_then(|v| v.as_i64()) {
                        if count > 0 {
                            return HttpResponse::BadRequest().json(WebServiceResponse {
                                status: "Error".into(),
                                info: "Username sudah terdaftar".into(),
                            });
                        }
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: format!("Gagal memeriksa username: {}", e),
            });
        }
    }

    let query_max_id = "SELECT max(id_user) as max_id FROM kerjasama.user FORMAT JSON".to_string();
    
    let id_user = match query_ch(query_max_id).await {
        Ok(result) => {
            if let Some(data_arr) = result.get("data").and_then(|d| d.as_array()) {
                if let Some(first_row) = data_arr.first() {
                    if let Some(max_id) = first_row.get("max_id").and_then(|v| v.as_i64()) {
                        (max_id + 1) as i32
                    } else {
                        1
                    }
                } else {
                    1
                }
            } else {
                1
            }
        }
        Err(_) => 1
    };

    let query = format!(
        "INSERT INTO kerjasama.user (id_user, username, email, fullname, password, role, nip, direktorat, jabatan, ttl, status, timestamp) VALUES ({}, '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', 1, now())",
        id_user,
        username.replace("'", "''"),
        email.replace("'", "''"),
        data.full_name.clone().unwrap_or_default().replace("'", "''"),
        bcrypt_password.replace("'", "''"),
        data.role.replace("'", "''"),
        data.nip.unwrap_or(0),
        data.direktorat.clone().unwrap_or_default().replace("'", "''"),
        data.jabatan.clone().unwrap_or_default().replace("'", "''"),
        data.ttl.clone().unwrap_or_default().replace("'", "''")
    );

    println!("📝 Registering user: {}", username);
    println!("   Email: {}", email);
    println!("   ID: {}", id_user);

    match query_ch(query).await {
        Ok(_) => {
            println!("✅ User registered successfully: {}", username);
            HttpResponse::Ok().json(WebServiceResponse {
                status: "Ok".into(),
                info: format!("🎉 Registrasi berhasil! Akun {} telah aktif dan siap digunakan.", username),
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to register user: {}", e);
            HttpResponse::InternalServerError().json(WebServiceResponse {
                status: "Error".into(),
                info: format!("Gagal mendaftarkan akun: {}", e),
            })
        }
    }
}

pub async fn verify_account(_token: web::Path<String>) -> impl Responder {
    HttpResponse::NotImplemented().json(WebServiceResponse {
        status: "Error".into(),
        info: "Fitur verifikasi email saat ini tidak aktif. Registrasi langsung aktif.".into(),
    })
}