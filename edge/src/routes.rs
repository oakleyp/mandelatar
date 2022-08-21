use log::error;
use worker::{Result as WorkerResult, *};

use crate::errors;
use crate::errors::ResponseError;
use crate::post_processing;
use mandelatar_core::image_params;
use mandelatar_core::mandelbrot;

const MAX_B64_LEN: usize = 500;

type ApiResult<T, E> = std::result::Result<T, E>;

fn parse_image_q_params(
    req: &Request,
) -> std::result::Result<image_params::ImagePostProcessConfig, errors::UserError> {
    let url = req
        .url()
        .map_err(|e| errors::UserError::WorkerError { error: e })?;

    let query_pairs = url
        .query_pairs()
        .into_owned()
        .collect::<Vec<(String, String)>>();

    image_params::ImagePostProcessConfig::from_query_params(&query_pairs).map_err(|_e| {
        errors::UserError::ValidationError {
            message: "Invalid query params provided".to_string(),
        }
    })
}

fn add_cors_headers(
    og_headers: &worker::Headers,
    req_headers: &worker::Headers,
    cors_origin: &str,
) -> ApiResult<worker::Headers, errors::UserError> {
    let mut result_headers = og_headers.to_owned();

    let origin = match req_headers.get("Origin").unwrap() {
        Some(value) => value,
        None => return Ok(result_headers),
    };

    result_headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    result_headers.set("Access-Control-Allow-Methods", "GET")?;
    result_headers.set("Vary", "Origin")?;

    for origin_element in cors_origin.split(',') {
        if origin.eq(origin_element) {
            result_headers.set("Access-Control-Allow-Origin", &origin)?;
            break;
        }
    }
    result_headers.set("Access-Control-Max-Age", "86400")?;

    Ok(result_headers)
}

fn preflight_get_response(
    headers: &worker::Headers,
    cors_origin: &str,
) -> ApiResult<Response, errors::UserError> {
    let default_headers = worker::Headers::new();
    let headers = add_cors_headers(&default_headers, headers, cors_origin)?;

    Ok(Response::empty()
        .unwrap()
        .with_headers(headers)
        .with_status(204))
}

fn get_random_image<D>(
    req: Request,
    ctx: RouteContext<D>,
) -> ApiResult<Response, errors::UserError> {
    let img_params = image_params::ImageParams::new_from_rand((
        image_params::OUTPUT_WIDTH,
        image_params::OUTPUT_HEIGHT,
    ));

    let imp_bin = bincode::serialize(&img_params).map_err(|e| {
        error!("Failed to serialize img params: {}", e);
        errors::UserError::InternalError
    })?;

    let b64 = base64::encode_config(imp_bin, base64::URL_SAFE);

    let mut new_url = req.url().map_err::<errors::UserError, _>(|e| e.into())?;
    new_url.set_path(format!("i1/i/{}.png", b64).as_str());

    let resp = Response::redirect(new_url).map_err::<errors::UserError, _>(|e| e.into())?;
    let headers = add_cors_headers(
        resp.headers(),
        req.headers(),
        &ctx.var("CORS_ORIGIN")?.to_string(),
    )?;

    Ok(resp.with_headers(headers))
}

async fn get_image<D>(
    req: Request,
    ctx: RouteContext<D>,
) -> ApiResult<Response, errors::UserError> {
    let img_b64: String = ctx
        .param("img_b64")
        .ok_or(errors::UserError::ValidationError {
            message: "invalid base64 provided".to_string(),
        })?
        .to_owned();

    if img_b64.trim().is_empty() || img_b64.len() > MAX_B64_LEN {
        return Err(errors::UserError::ValidationError {
            message: "invalid base64 provided".to_string(),
        });
    }

    let img_b64 = img_b64.replace(".png", "");

    let bin = base64::decode_config(img_b64, base64::URL_SAFE).map_err(|e| {
        error!("Failed to decode from b64: {}", e);
        errors::UserError::ValidationError {
            message: "Invalid base64 provided".to_string(),
        }
    })?;

    let img_params: image_params::ImageParams = bincode::deserialize(&bin).map_err(|e| {
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
        let kv_store = ctx.kv("MANDELATAR_ASSETS")?;
        png_bytes = post_processing::process_from_params(q_params, &mut png_bytes, &kv_store)
            .await
            .map_err(|e| {
                error!("Post processing failed: {}", e);
                errors::UserError::InternalError
            })?;
    }

    let resp = Response::from_bytes(png_bytes)?;
    let mut headers = resp.headers().to_owned();
    headers
        .set("content-type", "image/png")
        .map_err::<errors::UserError, _>(|e| e.into())?;

    headers = add_cors_headers(
        &headers,
        req.headers(),
        &ctx.var("CORS_ORIGIN")?.to_string(),
    )?;

    Ok(resp.with_headers(headers))
}

fn to_worker_result(res: ApiResult<Response, errors::UserError>) -> WorkerResult<Response> {
    match res {
        Ok(r) => Ok(r),
        Err(e) => match e {
            errors::UserError::WorkerError { error } => Err(error),
            _ => e.error_response(),
        },
    }
}

pub fn apply_routes<'a, D: 'a>(router: Router<'a, D>) -> Router<'a, D> {
    router
        .get("/i1/random", |req, ctx| {
            to_worker_result(get_random_image(req, ctx))
        })
        .options("/i1/random", |req, ctx| {
            to_worker_result(preflight_get_response(
                req.headers(),
                &ctx.var("CORS_ORIGIN")?.to_string(),
            ))
        })
        .get_async("/i1/i/:img_b64", |req, ctx| async move {
            to_worker_result(get_image(req, ctx).await)
        })
        .options("/i1/i/:img_b64", |req, ctx| {
            to_worker_result(preflight_get_response(
                req.headers(),
                &ctx.var("CORS_ORIGIN")?.to_string(),
            ))
        })
}
