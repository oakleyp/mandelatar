use criterion::{black_box, criterion_group, criterion_main, Criterion};

use enumflags2::make_bitflags;
use image::ImageFormat;
use mandelatar_core::image_params::{
    ImageParams, ImageTransformFlags, OUTPUT_HEIGHT, OUTPUT_WIDTH,
};
use mandelatar_core::mandelbrot::create_png;
use mandelatar_core::post_processing;

pub fn bench_image_no_post_proc(c: &mut Criterion) {
    let mut iparams = ImageParams::new_from_rand((OUTPUT_WIDTH, OUTPUT_HEIGHT));
    iparams.transform_flags = make_bitflags!(ImageTransformFlags::{ROT180 | HUEROT90});

    c.bench_function("create random png", |b| {
        b.iter(|| create_png(black_box(&iparams)))
    });
}

pub fn bench_image_post_proc_profile_overlay(c: &mut Criterion) {
    let mut iparams = ImageParams::new_from_rand((OUTPUT_WIDTH, OUTPUT_HEIGHT));
    iparams.transform_flags = make_bitflags!(ImageTransformFlags::{ROT180 | HUEROT90});

    let img = create_png(&iparams).unwrap();
    let overlay_buf = std::fs::read("assets/profile_overlay_300x300.png").unwrap();

    c.bench_function("rewrite image with overlay", |b| {
        b.iter(|| {
            let mut base =
                post_processing::load_img_from_buffer("base", &img, ImageFormat::Png).unwrap();
            let top = post_processing::load_img_from_buffer(
                "profile_overlay",
                black_box(&overlay_buf),
                ImageFormat::Png,
            )
            .unwrap();
            post_processing::process_overlay(black_box(&mut base), black_box(&top));
            post_processing::encode_result_png(black_box(&base.into_rgba8())).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_image_no_post_proc,
    bench_image_post_proc_profile_overlay
);
criterion_main!(benches);
