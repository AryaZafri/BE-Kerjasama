use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub status: String,
    pub info: String,
    pub data: Option<T>
}

#[derive(Serialize, Deserialize)]
pub struct ResponseWithPagination<T> {
    pub status: String,
    pub info: String,
    pub data: Option<T>,
    pub total_row: i64
}

#[derive(Serialize)]
pub struct WebServiceResponse {
    pub status: String,
    pub info: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
}

// ----------------------------------------------- AUTH
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SigninRequest {
    pub password: Option<String>,
    // pub username: Option<String>,
    // pub email: Option<String>,
    // pub phonenumber: Option<String>,
    pub input: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateUniqueCodeRequest {
    pub phonenumber: String,
    pub unique_code: String,
}

#[derive(Deserialize)]
pub struct ResendEmailRequest {
    pub phonenumber: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserAccess {
    pub id_user: i32,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub phonenumber: Option<String>,
    pub role: Option<String>,
    pub dashboard: Option<i8>,
    pub report: Option<i8>,
    pub commercial: Option<i8>,
    pub user_management: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAccessRegister {
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub confirm_pass: String,
    pub role: String,
    pub nip: Option<i32>,
    pub direktorat: Option<String>,
    pub jabatan: Option<String>,
    pub ttl: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub phonenumber: String,
    pub role: String,
    pub dashboard: Option<i8>,
    pub report: Option<i8>,
    pub commercial: Option<i8>,
    pub user_management: Option<i8>,
    // pub iat: u64,
    // pub exp: u64,
    pub iat_formatted: String,
    pub exp_formatted: String,
}

#[derive(Deserialize)]
pub struct ForgotRequest {
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub new_pass: String,
    pub confirm_pass: String,
    pub email: Option<String>,
    pub phonenumber: Option<String>,
    pub hash_question: Option<String>,
    pub hash_answer: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePassRequest {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Deserialize)]
pub struct UpdateAccountRequest {
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phonenumber: Option<String>,
    pub old_password: Option<String>,
    pub new_password: Option<String>,
    pub confirm_password: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct EditProfileRequest {
    pub old_password: Option<String>,
    pub new_password: Option<String>,
    pub confirm_password: Option<String>,
    pub username: Option<String>,
}
//---------------------------------REGISTER
#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub username: String,
    pub full_name: String,
    pub password: String,
    pub confirm_pass: String,
    pub email: String,
    pub role: String,
    pub exp: usize, // expiration time (dalam detik Unix epoch)
}

#[derive(Deserialize)]
pub struct ParamsPlatform {
    pub operator: String,
    pub platform: String,
}

#[derive(Deserialize)]
pub struct UserManagement {
    pub username: Option<String>,
    pub role: Option<String>,
    pub dashboard: Option<bool>,
    pub report: Option<bool>,
    pub commercial: Option<bool>,
    pub user_management: Option<bool>,
    pub status: Option<bool>,
}

#[derive(Deserialize)]
pub struct SortCondition {
    pub field: String,
    pub sort: String,
}