use mandelatar_core::errors;
use mandelatar_core::image_params::{ImagePostProcessConfig, OverlayImageTypes};
use mandelatar_core::post_processing as core_pp;

use image::ImageFormat;
use worker::kv;

pub async fn process_from_params(
    pp_config: ImagePostProcessConfig,
    image: &mut [u8],
    kv_store: &kv::KvStore,
) -> Result<Vec<u8>, errors::ImagePostProcessingError> {
    let mut base = core_pp::load_img_from_buffer("base image", image, ImageFormat::Png)?;

    if let Some(OverlayImageTypes::Profile { width, height }) = pp_config.overlay_image_type {
        let profile_overlay_buf = match (width, height) {
            (300, 300) => load_img_buf_from_kv_store(kv_store, "profile_overlay_300x300").await?,
            _ => load_img_buf_from_kv_store(kv_store, "profile_overlay_600x600").await?,
        };

        let top = core_pp::load_img_from_buffer(
            "profile_overlay",
            &profile_overlay_buf,
            ImageFormat::Png,
        )?;

        core_pp::process_overlay(&mut base, &top);
    }

    core_pp::encode_result_png(&base.to_rgba8())
}

pub async fn load_img_buf_from_kv_store(
    kv_store: &kv::KvStore,
    name: &str,
) -> Result<Vec<u8>, errors::ImagePostProcessingError> {
    kv_store
        .get(name)
        .bytes()
        .await
        .map_err(|e| errors::ImagePostProcessingError::Default {
            message: format!("failed to fetch {} from kv store: {}", name, e),
        })?
        .ok_or(errors::ImagePostProcessingError::Default {
            message: format!("{} buffer was empty", name),
        })
}
