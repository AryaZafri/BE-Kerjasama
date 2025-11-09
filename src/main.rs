use actix_cors::Cors;
use actix_web::{error, http::header, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use kerjasama::config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let port = std::env::var("SERVER_PORT")
        .expect("no environment variable set for \"ENV STATUS\"")
        .parse::<u16>()
        .unwrap_or(8008);
    println!("Starting server on port {}", port);

    let _ = HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        let json_cfg = web::JsonConfig::default()
            .limit(104857600)
            .error_handler(|err, _req| {
                error::InternalError::from_response(err, HttpResponse::Conflict().into()).into()
            });

        App::new().wrap(cors).configure(config).app_data(json_cfg)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}