use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use actix_service::{Service, Transform};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::pin::Pin;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareMiddleware {
            service: Arc::new(service),
        })
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let auth_header = req.headers().get("Authorization").cloned();
        let service = Arc::clone(&self.service);

        Box::pin(async move {
            println!("Request path: {}", path);

            // Public paths yang tidak memerlukan autentikasi
            if path.starts_with("/login") 
                || path.starts_with("/register") 
                || path.starts_with("/verify")
                || path.starts_with("/logout") 
                || path.starts_with("/forgot_pass") 
                || path.starts_with("/edit_profile") 
                || path.starts_with("/validate_code")
                || path.starts_with("/resend_verification_email") 
                || path.starts_with("/change_pass")
            {
                println!("Public path accessed: {}", path);
                return service.call(req).await;
            }

            // Validasi Authorization header
            if let Some(auth_value) = auth_header {
                if let Ok(auth_str) = auth_value.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        println!("Token extracted: {}", token);

                        if validate_token(token).await {
                            println!("Token is valid");
                            return service.call(req).await;
                        } else {
                            println!("Token is invalid");
                        }
                    }
                }
            }

            println!("Unauthorized request");
            Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
        })
    }
}

async fn validate_token(token: &str) -> bool {
    use reqwest::{redirect::Policy, ClientBuilder};
    use serde_json::Value;

    let host = match std::env::var("CH_HOST") {
        Ok(h) => h,
        Err(_) => {
            println!("CH_HOST not found in environment");
            return false;
        }
    };
    
    let port = match std::env::var("CH_PORT") {
        Ok(p) => p,
        Err(_) => {
            println!("CH_PORT not found in environment");
            return false;
        }
    };
    
    let username = match std::env::var("CH_USERNAME") {
        Ok(u) => u,
        Err(_) => {
            println!("CH_USERNAME not found in environment");
            return false;
        }
    };
    
    let password = match std::env::var("CH_PASSWORD") {
        Ok(p) => p,
        Err(_) => {
            println!("CH_PASSWORD not found in environment");
            return false;
        }
    };

    let url = format!(
        "http://{}:{}/?user={}&password={}",
        host, port, username, password
    );

    let query = format!(
        "SELECT count() as cnt FROM db_kerjasama.user WHERE token = '{}' AND status = 1 FORMAT JSON",
        token.replace("'", "''")
    );

    let client = match ClientBuilder::new()
        .cookie_store(true)
        .danger_accept_invalid_certs(true)
        .redirect(Policy::limited(20))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to create HTTP client: {}", e);
            return false;
        }
    };

    let res = match client
        .post(&url)
        .header("Content-Type", "text/plain")
        .body(query)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to send request to ClickHouse: {}", e);
            return false;
        }
    };

    let text = match res.text().await {
        Ok(t) => t,
        Err(e) => {
            println!("Failed to read response text: {}", e);
            return false;
        }
    };

    match serde_json::from_str::<Value>(&text) {
        Ok(json) => {
            if let Some(data_arr) = json.get("data").and_then(|d| d.as_array()) {
                if let Some(first_row) = data_arr.first() {
                    if let Some(count) = first_row.get("cnt").and_then(|v| v.as_i64()) {
                        return count > 0;
                    }
                }
            }
            false
        }
        Err(e) => {
            println!("Failed to parse JSON response: {}", e);
            false
        }
    }
}