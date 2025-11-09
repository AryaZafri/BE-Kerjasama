use actix_web::{web, HttpResponse, Responder, cookie::Cookie};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use time::{Duration, OffsetDateTime};

use crate::query_ch;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub status: String,
    pub info: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<UserInfo>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id_user: i32,
    pub username: String,
    pub email: String,
    pub fullname: String,
    pub role: String,
    pub nip: Option<i64>,
    pub direktorat: Option<String>,
    pub jabatan: Option<String>,
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
const TOKEN_EXPIRATION: i64 = 24 * 60 * 60;

pub async fn login(data: web::Json<LoginRequest>) -> impl Responder {
    let username = data.username.trim();
    let password = &data.password;

    if username.is_empty() || password.is_empty() {
        return HttpResponse::BadRequest().json(LoginResponse {
            status: "Error".into(),
            info: "Username dan password harus diisi".into(),
            data: None,
        });
    }

    let query = format!(
        "SELECT id_user, username, email, fullname, password, role, nip, direktorat, jabatan, status FROM kerjasama.user WHERE username = '{}' FORMAT JSON",
        username.replace("'", "''")
    );

    println!("🔐 Login attempt for username: {}", username);

    let user_data = match query_ch(query).await {
        Ok(result) => {
            if let Some(data_arr) = result.get("data").and_then(|d| d.as_array()) {
                if let Some(user_row) = data_arr.first() {
                    user_row.clone()
                } else {
                    return HttpResponse::Unauthorized().json(LoginResponse {
                        status: "Error".into(),
                        info: "Username atau password salah".into(),
                        data: None,
                    });
                }
            } else {
                return HttpResponse::Unauthorized().json(LoginResponse {
                    status: "Error".into(),
                    info: "Username atau password salah".into(),
                    data: None,
                });
            }
        }
        Err(e) => {
            eprintln!("❌ Database error during login: {}", e);
            return HttpResponse::InternalServerError().json(LoginResponse {
                status: "Error".into(),
                info: "Terjadi kesalahan pada server".into(),
                data: None,
            });
        }
    };

    let status = user_data.get("status").and_then(|v| v.as_i64()).unwrap_or(0);
    if status != 1 {
        return HttpResponse::Forbidden().json(LoginResponse {
            status: "Error".into(),
            info: "Akun Anda belum aktif atau telah dinonaktifkan".into(),
            data: None,
        });
    }

    let hashed_password = user_data
        .get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match bcrypt::verify(password, hashed_password) {
        Ok(valid) => {
            if !valid {
                println!("❌ Invalid password for user: {}", username);
                return HttpResponse::Unauthorized().json(LoginResponse {
                    status: "Error".into(),
                    info: "Username atau password salah".into(),
                    data: None,
                });
            }
        }
        Err(e) => {
            eprintln!("❌ Error verifying password: {}", e);
            return HttpResponse::InternalServerError().json(LoginResponse {
                status: "Error".into(),
                info: "Terjadi kesalahan pada server".into(),
                data: None,
            });
        }
    }

    let id_user = user_data.get("id_user").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let email = user_data.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let fullname = user_data.get("fullname").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let role = user_data.get("role").and_then(|v| v.as_str()).unwrap_or("user").to_string();
    let nip = user_data.get("nip").and_then(|v| v.as_i64());
    let direktorat = user_data.get("direktorat").and_then(|v| v.as_str()).map(String::from);
    let jabatan = user_data.get("jabatan").and_then(|v| v.as_str()).map(String::from);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: id_user,
        username: username.to_string(),
        email: email.clone(),
        role: role.clone(),
        exp: now + TOKEN_EXPIRATION as usize,
        iat: now,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Error generating token: {}", e);
            return HttpResponse::InternalServerError().json(LoginResponse {
                status: "Error".into(),
                info: "Gagal membuat token autentikasi".into(),
                data: None,
            });
        }
    };

    println!("✅ Login successful for user: {} (ID: {})", username, id_user);

    let expiration = OffsetDateTime::now_utc() + Duration::seconds(TOKEN_EXPIRATION);
    
    let cookie = Cookie::build("auth_token", token)
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(actix_web::cookie::SameSite::Lax)
        .expires(expiration)
        .finish();

    let mut response = HttpResponse::Ok().json(LoginResponse {
        status: "Ok".into(),
        info: "Login berhasil".into(),
        data: Some(UserInfo {
            id_user,
            username: username.to_string(),
            email,
            fullname,
            role,
            nip,
            direktorat,
            jabatan,
        }),
    });

    response.add_cookie(&cookie).unwrap();
    response
}

pub async fn logout_service() -> impl Responder {
    let _expiration = chrono::Utc::now();
    let now = OffsetDateTime::now_utc();

    let mut cookie = Cookie::build("auth_token", "")
        .path("/")
        .expires(now)
        .finish();

    cookie.set_expires(OffsetDateTime::now_utc());
    cookie.set_expires(None);

    let mut response = HttpResponse::Ok().body("Logout successfully");
    response.add_cookie(&cookie).unwrap();

    response
}