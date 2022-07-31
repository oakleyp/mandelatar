use image::codecs::png::PngEncoder;
use image::ColorType;
use image::ImageEncoder;
use image::Rgb;
use log::debug;
use num::Complex;
use rand;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub const OUTPUT_WIDTH: usize = 600;
pub const OUTPUT_HEIGHT: usize = 600;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Complex::<f64>")]
struct ComplexDef {
    re: f64,
    im: f64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageParams {
    bounds: (usize, usize),
    #[serde(with = "ComplexDef")]
    upper_left: Complex<f64>,
    #[serde(with = "ComplexDef")]
    lower_right: Complex<f64>,
    zoom_factor: f64,
    rgb_consts: (u8, u8, u8),
}

impl ImageParams {
    fn get_bounds(&self) -> (usize, usize) {
        // Restrict changing from default bounds, for now
        (OUTPUT_WIDTH, OUTPUT_HEIGHT)
    }

    fn get_relative_point(pixel: f64, length: f64, set: (f64, f64)) -> f64 {
        let (start, end) = set;
        start + (pixel / length) * (end - start)
    }

    // Given a set of image bounds, create a random set of ImageParams
    pub fn new_from_rand(bounds: (usize, usize)) -> Self {
        let mut rng = rand::thread_rng();

        // These points were chosen as an "interesting" selection of the set
        // to start from.
        // The list of possible starting points should be expanded for this
        // randomization procedure in the future.
        let mut upper_left = Complex::new(-1.20, 0.35);
        let mut lower_right = Complex::new(-1.0, 0.20);

        let exp = rng.gen_range(1..10);
        let zoom_factor = 1.0 / 10.0_f64.powi(exp) * rng.gen_range(1.0..9.0);

        debug!("zoom factor {} - {}", exp, zoom_factor);

        let zfw = OUTPUT_WIDTH as f64 * zoom_factor;
        let zfh = OUTPUT_HEIGHT as f64 * zoom_factor;

        // Randomly choose a pixel to zoom from
        let middle_px_x: f64 = OUTPUT_WIDTH as f64 / 2.0 + rng.gen_range(-20.0..50.0);
        let middle_px_y: f64 = OUTPUT_HEIGHT as f64 / 2.0 + rng.gen_range(-30.0..50.0);
        let offset_left: f64 = 0.0;
        let offset_top: f64 = 0.0;

        upper_left.re = Self::get_relative_point(
            middle_px_x - offset_left - zfw,
            OUTPUT_WIDTH as f64,
            (upper_left.re, lower_right.re),
        );
        lower_right.re = Self::get_relative_point(
            middle_px_x - offset_top + zfw,
            OUTPUT_WIDTH as f64,
            (upper_left.re, lower_right.re),
        );

        upper_left.im = Self::get_relative_point(
            middle_px_y - offset_top - zfh,
            OUTPUT_HEIGHT as f64,
            (upper_left.im, lower_right.im),
        );
        lower_right.im = Self::get_relative_point(
            middle_px_y - offset_top + zfh,
            OUTPUT_HEIGHT as f64,
            (upper_left.im, lower_right.im),
        );

        let rgb_consts = (
            rng.gen_range::<u8, _>(0..255),
            rng.gen_range::<u8, _>(0..255),
            rng.gen_range::<u8, _>(0..255),
        );

        Self {
            bounds,
            upper_left,
            lower_right,
            zoom_factor,
            rgb_consts,
        }
    }
}

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

    let mut buffer = vec![];

    let encoder = PngEncoder::new(&mut buffer);

    let pb_flat: Vec<u8> = pixels
        .iter()
        .flat_map(|rgb| rgb.0.iter())
        .cloned()
        .collect();

    encoder
        .write_image(
            &pb_flat,
            OUTPUT_WIDTH as u32,
            OUTPUT_HEIGHT as u32,
            ColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;

    Ok(buffer.to_vec())
}