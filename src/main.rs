mod errors;

use actix_web::{get, http::header, http::StatusCode, web, App, HttpResponse, HttpServer};
use log::{error, info};
use mandelib::mandelbrot;

use base64;
use bincode;
use env_logger::Env;
use num::Complex;

#[get("/v1/random")]
async fn get_random() -> Result<HttpResponse, errors::UserError> {
    let img_params = mandelbrot::ImageParams::new_from_rand((
        mandelbrot::OUTPUT_WIDTH,
        mandelbrot::OUTPUT_HEIGHT,
    ));
    let imp_bin = bincode::serialize(&img_params).map_err(|e| {
        error!("Failed to serialize img params: {}", e);
        errors::UserError::InternalError
    })?;

    let b64 = base64::encode_config(imp_bin, base64::URL_SAFE);

    Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .insert_header((header::LOCATION, format!("/v1/{}", b64)))
        .finish())
}

#[get("/v1/{img_hash}")]
async fn get_image(path: web::Path<String>) -> Result<HttpResponse, errors::UserError> {
    let img_hash: String = path.into_inner();

    if img_hash.trim().is_empty() {
        return Err(errors::UserError::ValidationError {
            message: "invalid base64 provided".to_string(),
        });
    }

    let bin = base64::decode_config(img_hash, base64::URL_SAFE).map_err(|e| {
        error!("Failed to decode from b64: {}", e);
        errors::UserError::ValidationError {
            message: "Invalid base64 provided".to_string(),
        }
    })?;

    let img_params: mandelbrot::ImageParams = bincode::deserialize(&bin).map_err(|e| {
        error!("Failed to deserialize from b64: {}", e);
        errors::UserError::ValidationError {
            message: "Invalid base64 provided".to_string(),
        }
    })?;

    let png_bytes = mandelbrot::create_png(&img_params).map_err(|e| {
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

    HttpServer::new(|| App::new().service(get_random).service(get_image))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
