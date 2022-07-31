use enumflags2::{bitflags, make_bitflags, BitFlags};
use log::debug;
use num::Complex;
use rand;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const OUTPUT_WIDTH: usize = 600;
pub const OUTPUT_HEIGHT: usize = 600;

// Remote-derive serial/deserialize for foreign type num::Complex
// Potential TODO: Make generic on Complex<T>
#[derive(Serialize, Deserialize)]
#[serde(remote = "Complex::<f64>")]
struct ComplexDef {
    re: f64,
    im: f64,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImageTransformFlags {
    ROT180 = 0b0001,
    HUEROT90,
    INVERT,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageParams {
    pub bounds: (usize, usize),
    #[serde(with = "ComplexDef")]
    pub upper_left: Complex<f64>,
    #[serde(with = "ComplexDef")]
    pub lower_right: Complex<f64>,
    pub zoom_factor: f64,
    pub rgb_consts: (u8, u8, u8),
    pub transform_flags: BitFlags<ImageTransformFlags>,
}

impl ImageParams {
    pub fn get_bounds(&self) -> (usize, usize) {
        // Restrict changing from default bounds, for now
        (OUTPUT_WIDTH, OUTPUT_HEIGHT)
    }

    fn get_relative_point(pixel: f64, length: f64, set: (f64, f64)) -> f64 {
        let (start, end) = set;
        start + (pixel / length) * (end - start)
    }

    fn enabled_transforms() -> Vec<ImageTransformFlags> {
        vec![
            ImageTransformFlags::ROT180,
            ImageTransformFlags::HUEROT90,
            // ImageTransformFlags::INVERT,
        ]
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

        let random_transform_flags = Self::enabled_transforms().iter().fold(
            BitFlags::EMPTY,
            |acc: BitFlags<_, _>, flag: &ImageTransformFlags| {
                if rng.gen_bool(0.5) {
                    return acc | flag.to_owned();
                }

                acc
            },
        );

        Self {
            bounds,
            upper_left,
            lower_right,
            zoom_factor,
            rgb_consts,
            transform_flags: random_transform_flags,
        }
    }
}
