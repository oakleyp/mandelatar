mod errors;
mod server_config;

use actix_cors::Cors;
use actix_web::{
    get, http::header, http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer,
};
use env_logger::Env;
use log::{error, info};
use mandelatar_core::image_params::{
    ImageParams, ImagePostProcessConfig, OUTPUT_HEIGHT, OUTPUT_WIDTH,
};
use mandelatar_core::mandelbrot;
use mandelatar_core::post_processing;
use url::Url;

const MAX_B64_LEN: usize = 500;

fn parse_image_q_params(
    req: &HttpRequest,
) -> std::result::Result<ImagePostProcessConfig, errors::UserError> {
    let url = Url::parse(
        format!(
            "{}{}",
            req.url_for_static("random_image").unwrap(),
            req.uri()
        )
        .as_ref(),
    )
    .map_err(|e| {
        error!("Failed parsing url: {}", e);
        errors::UserError::InternalError
    })?;

    let query_pairs = url
        .query_pairs()
        .into_owned()
        .collect::<Vec<(String, String)>>();

    ImagePostProcessConfig::from_query_params(&query_pairs).map_err(|e| {
        error!("Invalid q params: {}", e);
        errors::UserError::ValidationError {
            message: "Invalid query params provided".to_string(),
        }
    })
}

#[get("/i1/random")]
async fn get_random_from_worker_failover() -> Result<HttpResponse, errors::UserError> {
    get_random().await
}

#[get("/api/v1/random", name = "random_image")]
async fn get_random_direct() -> Result<HttpResponse, errors::UserError> {
    get_random().await
}

async fn get_random() -> Result<HttpResponse, errors::UserError> {
    let img_params = ImageParams::new_from_rand((OUTPUT_WIDTH, OUTPUT_HEIGHT));
    let imp_bin = bincode::serialize(&img_params).map_err(|e| {
        error!("Failed to serialize img params: {}", e);
        errors::UserError::InternalError
    })?;

    let b64 = base64::encode_config(imp_bin, base64::URL_SAFE);

    Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .insert_header((header::LOCATION, format!("/api/v1/img/{}", b64)))
        .finish())
}

#[get("/i1/i/{img_b64}")]
async fn get_image_from_worker_failover(
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, errors::UserError> {
    get_image(path, req).await
}

#[get("/api/v1/img/{img_b64}", name = "get_image")]
async fn get_image_direct(
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, errors::UserError> {
    get_image(path, req).await
}

async fn get_image(
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, errors::UserError> {
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

    let img_params: ImageParams = bincode::deserialize(&bin).map_err(|e| {
        error!("Failed to deserialize from b64: {}", e);
        errors::UserError::ValidationError {
            message: "Invalid base64 provided".to_string(),
        }
    })?;

    let mut png_bytes = mandelbrot::create_png(&img_params).map_err(|e| {
        error!("Failed to create image: {}", e);
        errors::UserError::InternalError
    })?;

    let q_params = parse_image_q_params(&req)?;

    if q_params.should_post_process() {
        png_bytes =
            post_processing::process_from_params(&q_params, &mut png_bytes).map_err(|e| {
                error!("Post processing failed: {}", e);
                errors::UserError::InternalError
            })?;
    }

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/png")
        .body(png_bytes))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let env = Env::default();

    env_logger::init_from_env(env);
    let args = server_config::ServerConfig::load_from_env();

    let server_port = args.server_port;
    let server_addr = args.server_addr.to_owned();

    info!("Server started :)");

    HttpServer::new(move || {
        let args = args.clone();
        let cors = Cors::default()
            .allowed_origin_fn(move |origin, _req_head| {
                for accept_origin in &args.cors_origins {
                    if accept_origin.trim().is_empty() {
                        continue;
                    }

                    if origin.as_bytes().ends_with(accept_origin.as_bytes()) {
                        return true;
                    }
                }

                false
            })
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(get_random_direct)
            .service(get_random_from_worker_failover)
            .service(get_image_direct)
            .service(get_image_from_worker_failover)
    })
    .bind((server_addr, server_port))?
    .run()
    .await?;

    Ok(())
}
