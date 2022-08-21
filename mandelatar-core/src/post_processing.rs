use crate::errors;
use crate::image_params::{ImagePostProcessConfig, OverlayImageTypes, OUTPUT_HEIGHT, OUTPUT_WIDTH};

use image::codecs::png::PngEncoder;
use image::DynamicImage;
use image::GenericImageView;
use image::{self, imageops, ColorType, GenericImage, ImageEncoder, ImageFormat, Pixel};

pub fn process_from_params(
    q_params: &ImagePostProcessConfig,
    image: &mut [u8],
) -> Result<Vec<u8>, errors::ImagePostProcessingError> {
    let mut base = load_img_from_buffer("base image", image, ImageFormat::Png)?;

    if let Some(overlay_image_type) = q_params.overlay_image_type {
        match overlay_image_type {
            OverlayImageTypes::Profile { width, height } => {
                let profile_overlay_buf = match (width, height) {
                    (300, 300) => include_bytes!("../assets/profile_overlay_300x300.png").to_vec(),
                    _ => include_bytes!("../assets/profile_overlay_600x600.png").to_vec(),
                };

                let top = load_img_from_buffer(
                    "profile_overlay",
                    &profile_overlay_buf,
                    ImageFormat::Png,
                )?;

                process_overlay(&mut base, &top);
            }
        }
    }

    encode_result_png(&base.to_rgba8())
}

pub fn load_img_from_buffer(
    name: &str,
    buf: &[u8],
    format: ImageFormat,
) -> Result<DynamicImage, errors::ImagePostProcessingError> {
    image::load_from_memory_with_format(buf, format).map_err(|e| {
        errors::ImagePostProcessingError::Default {
            message: format!("failed to load image {} from buffer: {}", name, e),
        }
    })
}

pub fn process_overlay<I, J>(base: &mut I, top: &J)
where
    I: GenericImage,
    J: GenericImageView<Pixel = I::Pixel>,
    J::Pixel: 'static,
    <<J as GenericImageView>::Pixel as Pixel>::Subpixel: 'static,
{
    let (dim_w, dim_h) = top.dimensions();

    // Resize overlay image if dimensions do not match
    if dim_w != OUTPUT_WIDTH as u32 || dim_h != OUTPUT_HEIGHT as u32 {
        // Note: FilterType `Nearest` is chosen as the fastest here, although resolution is better on "Gaussian"
        let top = imageops::resize(
            top,
            OUTPUT_WIDTH as u32,
            OUTPUT_HEIGHT as u32,
            imageops::FilterType::Nearest,
        );
        imageops::overlay(base, &top, 0, 0);
    } else {
        imageops::overlay(base, top, 0, 0);
    }
}

pub fn encode_result_png(buf: &[u8]) -> Result<Vec<u8>, errors::ImagePostProcessingError> {
    let mut buffer = vec![];
    let encoder = PngEncoder::new(&mut buffer);

    // Write image_buffer to result buffer
    encoder
        .write_image(
            buf,
            OUTPUT_WIDTH as u32,
            OUTPUT_HEIGHT as u32,
            ColorType::Rgba8,
        )
        .map_err(|e| errors::ImagePostProcessingError::Default {
            message: format!("failed to write to result image buffer: {}", e),
        })?;

    Ok(buffer.to_vec())
}
