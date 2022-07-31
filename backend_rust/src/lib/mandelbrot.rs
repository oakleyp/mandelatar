use crate::imageparams::{ImageParams, ImageTransformFlags, OUTPUT_HEIGHT, OUTPUT_WIDTH};
use image::codecs::png::PngEncoder;
use image::imageops;
use image::ColorType;
use image::ImageBuffer;
use image::ImageEncoder;
use image::Rgb;
use image::RgbImage;
use num::Complex;
use rayon::prelude::*;

/// Try to determine if `c` is in the Mandelbrot set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius two centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };

    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }

        z = z * z + c;
    }

    None
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
///
/// `bounds` is a pair giving the width and height of the image in pixels.
/// `pixel` is a (column, row) pair indicating a particular pixel in that image.
/// The `upper_left` and `lower_right` parameters are points on the complex
/// plane designating the area our image covers.
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );

    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
        // Why subtraction here? pixel.1 increases as we go down,
        // but the imaginary component increases as we go up.
    }
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The `bounds` argument gives the width and height of the buffer `pixels`,
/// which holds one grayscale pixel per byte. The `upper_left` and `lower_right`
/// arguments specify points on the complex plane corresponding to the upper-
/// left and lower-right corners of the pixel buffer.
fn render(
    pixels: &mut [Rgb<u8>],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
    (r, g, b): (u8, u8, u8),
) {
    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);

            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => Rgb([10, 10, 25]),
                Some(count) => Rgb([r % count as u8, g % count as u8, b % count as u8]),
            };
        }
    }
}

pub fn apply_image_transforms_in_place(
    img_params: &ImageParams,
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
) {
    let transform_flags = img_params.transform_flags;

    if transform_flags.contains(ImageTransformFlags::ROT180) {
        imageops::rotate180_in_place(image);
    }

    if transform_flags.contains(ImageTransformFlags::HUEROT90) {
        imageops::huerotate(image, 90);
    }

    if transform_flags.contains(ImageTransformFlags::INVERT) {
        imageops::invert(image);
    }
}

// Generate a PNG deterministically from the given set of `ImageParams`
pub fn create_png(img_params: &ImageParams) -> Result<Vec<u8>, String> {
    let img_bounds = img_params.get_bounds();
    let mut pixels = vec![Rgb([0, 0, 0]); img_bounds.0 * img_bounds.1];

    // Scope of slicing up `pixels` into horizontal bands for parallel processing
    {
        let bands: Vec<(usize, &mut [Rgb<u8>])> = pixels
            .chunks_mut(img_bounds.0)
            .enumerate()
            .collect::<Vec<(usize, &mut [Rgb<u8>])>>();

        bands.into_par_iter().for_each(|(i, band)| {
            let top = i;
            let band_bounds = (img_bounds.0, 1);
            let band_upper_left = pixel_to_point(
                img_bounds,
                (0, top),
                img_params.upper_left,
                img_params.lower_right,
            );
            let band_lower_right = pixel_to_point(
                img_bounds,
                (img_bounds.0, top + 1),
                img_params.upper_left,
                img_params.lower_right,
            );

            render(
                band,
                band_bounds,
                band_upper_left,
                band_lower_right,
                img_params.rgb_consts,
            );
        });
    }

    // Flatten 2d pixel array to 1d
    let pb_flat: Vec<u8> = pixels
        .iter()
        .flat_map(|rgb| rgb.0.iter())
        .cloned()
        .collect();

    // Rewrap pixels to support image operations
    let mut image_buffer: RgbImage =
        RgbImage::from_raw(img_bounds.0 as u32, img_bounds.1 as u32, pb_flat).ok_or(
            "Failed to convert pixel array to ImageBuffer (wrong size, somehow).".to_string(),
        )?;

    apply_image_transforms_in_place(&img_params, &mut image_buffer);

    // Result image buffer
    let mut buffer = vec![];
    let encoder = PngEncoder::new(&mut buffer);

    // Write image_buffer to result buffer
    encoder
        .write_image(
            &image_buffer,
            OUTPUT_WIDTH as u32,
            OUTPUT_HEIGHT as u32,
            ColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;

    Ok(buffer.to_vec())
}
