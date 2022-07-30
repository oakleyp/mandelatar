use actix_web::{
    error, get,
    http::{header::ContentType, StatusCode},
    web, App, HttpResponse, HttpServer,
};
use log::{error, info};
use mandelib;

use env_logger::Env;
use num::Complex;

#[derive(Debug)]
enum UserError {
    ValidationError { message: String },
    InternalError,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UserError::ValidationError { message } => write!(f, "Validation Error: {}", message),
            UserError::InternalError => write!(f, "An internal server error occurred"),
        }
    }
}

impl error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[get("/{img_hash}")]
async fn get_image(path: web::Path<String>) -> Result<HttpResponse, UserError> {
    let img_hash: String = path.into_inner();

    if img_hash.trim().is_empty() {
        return Err(UserError::ValidationError {
            message: "invalid hash provided".to_string(),
        });
    }

    let png_bytes = mandelib::create_png(
        (600, 600),
        Complex::new(-1.30, 0.35),
        Complex::new(-1.1, 0.20),
    )
    .map_err(|e| {
        error!("Failed to create image: {}", e);

        UserError::InternalError
    })?;

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/png")
        .body(png_bytes))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let env = Env::default();

    env_logger::init_from_env(env);

    info!("Server started :)");

    HttpServer::new(|| App::new().service(get_image))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
