use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub status: String,
    pub info: String,
    pub data: Option<T>
}

#[derive(Serialize)]
pub struct WebServiceResponse {
    pub status: String,
    pub info: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAccessRegister {
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub confirm_pass: String,
    pub role: Option<String>,
    pub nip: Option<String>,
    pub direktorat: Option<String>,
    pub jabatan: Option<String>,
    pub ttl: Option<String>,
}