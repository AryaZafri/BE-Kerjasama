use log::LevelFilter;
use chrono::Local;
use std::io::Write;
use std::fmt;
use actix_web::error::HttpError;

// ini utk inisialisasi logger saja
pub fn init_logger() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                     record.level(),
                     record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .filter(Some("sqlx"), LevelFilter::Warn)
        .init();
}

#[derive(Debug, serde::Deserialize)]
pub struct CustomError {
    pub error_status_code: u16,
    pub error_message: String,
}

impl CustomError {
    pub fn new(error_status_code: u16, error_message: String) -> CustomError {
        log::error!("{}", error_message);
        CustomError {
            error_status_code,
            error_message: "Internal Server Error".into(),
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error_message.as_str())
    }
}

impl From<std::io::Error> for CustomError {
    fn from(error: std::io::Error) -> CustomError {
        CustomError::new(500, format!("std io error: {}", error))
    }
}

impl From<HttpError> for CustomError {
    fn from(error: HttpError) -> CustomError {
        CustomError::new(500, format!("Http Error: {}", error))
    }
}

impl actix_web::ResponseError for CustomError {} // <-- key