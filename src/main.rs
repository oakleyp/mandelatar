mod errors;

use actix_web::{get, http::StatusCode, web, App, HttpResponse, HttpServer};
use log::{error, info};
use mandelib::mandelbrot;

use env_logger::Env;
use num::Complex;

#[get("/{img_hash}")]
async fn get_image(path: web::Path<String>) -> Result<HttpResponse, errors::UserError> {
    let img_hash: String = path.into_inner();

    if img_hash.trim().is_empty() {
        return Err(errors::UserError::ValidationError {
            message: "invalid hash provided".to_string(),
        });
    }

    let png_bytes = mandelbrot::create_png(
        (600, 600),
        Complex::new(-1.30, 0.35),
        Complex::new(-1.1, 0.20),
    )
    .map_err(|e| {
        error!("Failed to create image: {}", e);

        errors::UserError::InternalError
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
