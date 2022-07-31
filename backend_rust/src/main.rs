mod errors;
mod server_args;

use actix_web::{get, http::header, http::StatusCode, web, App, HttpResponse, HttpServer};
use log::{error, info};
use mandelib::mandelbrot;

use base64;
use bincode;
use env_logger::Env;

const MAX_B64_LEN: usize = 500;

#[get("/api/v1/random")]
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
        .insert_header((header::LOCATION, format!("/api/v1/{}", b64)))
        .finish())
}

#[get("/api/v1/{img_b64}")]
async fn get_image(path: web::Path<String>) -> Result<HttpResponse, errors::UserError> {
    let img_b64: String = path.into_inner();

    if img_b64.trim().is_empty() || img_b64.len() > MAX_B64_LEN {
        return Err(errors::UserError::ValidationError {
            message: "invalid base64 provided".to_string(),
        });
    }

    let bin = base64::decode_config(img_b64, base64::URL_SAFE).map_err(|e| {
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
    let args = server_args::ServerArgs::load_from_env();

    info!("Server started :)");

    HttpServer::new(|| App::new().service(get_random).service(get_image))
        .bind((args.server_addr, args.server_port))?
        .run()
        .await?;

    Ok(())
}
