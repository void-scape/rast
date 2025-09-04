use criterion::{Criterion, criterion_group, criterion_main};
use rast::prelude::*;
use std::hint::black_box;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn triangle2d(pixel_buffer: &mut PixelBuffer) {
    let v1 = Vec2::new(
        black_box(WIDTH as f32 / 3.0),
        black_box(HEIGHT as f32 / 3.0 * 2.0),
    );
    let v2 = Vec2::new(
        black_box(WIDTH as f32 / 2.0),
        black_box(HEIGHT as f32 / 3.0),
    );
    let v3 = Vec2::new(
        black_box(WIDTH as f32 / 3.0 * 2.0),
        black_box(HEIGHT as f32 / 3.0 * 2.0),
    );
    rast::rast_triangle2d(
        black_box(pixel_buffer),
        black_box(v1),
        black_box(v2),
        black_box(v3),
    );
}

fn triangle2d_rgb(pixel_buffer: &mut PixelBuffer) {
    let v1 = Vec2::new(
        black_box(WIDTH as f32 / 3.0),
        black_box(HEIGHT as f32 / 3.0 * 2.0),
    );
    let v2 = Vec2::new(
        black_box(WIDTH as f32 / 2.0),
        black_box(HEIGHT as f32 / 3.0),
    );
    let v3 = Vec2::new(
        black_box(WIDTH as f32 / 3.0 * 2.0),
        black_box(HEIGHT as f32 / 3.0 * 2.0),
    );

    let c1 = LinearRgb::rgb(black_box(1.0), black_box(0.0), black_box(0.0));
    let c2 = LinearRgb::rgb(black_box(0.0), black_box(1.0), black_box(0.0));
    let c3 = LinearRgb::rgb(black_box(0.0), black_box(0.0), black_box(1.0));

    rast::rast_triangle2d_shaded(
        black_box(pixel_buffer),
        black_box(v1),
        black_box(v2),
        black_box(v3),
        black_box(c1),
        black_box(c2),
        black_box(c3),
        black_box(ColorShader),
    );
}

fn triangle2d_texture(pixel_buffer: &mut PixelBuffer, texture: TextureShader<Srgb>) {
    let v1 = Vec2::new(
        black_box(WIDTH as f32 / 3.0),
        black_box(HEIGHT as f32 / 3.0 * 2.0),
    );
    let v2 = Vec2::new(
        black_box(WIDTH as f32 / 2.0),
        black_box(HEIGHT as f32 / 3.0),
    );
    let v3 = Vec2::new(
        black_box(WIDTH as f32 / 3.0 * 2.0),
        black_box(HEIGHT as f32 / 3.0 * 2.0),
    );

    let uv1 = Vec2::new(black_box(0.0), black_box(1.0));
    let uv2 = Vec2::new(black_box(0.0), black_box(0.0));
    let uv3 = Vec2::new(black_box(1.0), black_box(1.0));

    rast::rast_triangle2d_shaded(
        black_box(pixel_buffer),
        black_box(v1),
        black_box(v2),
        black_box(v3),
        black_box(uv1),
        black_box(uv2),
        black_box(uv3),
        black_box(texture),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut pixel_buffer = PixelBuffer::new(WIDTH, HEIGHT);
    c.bench_function("triangle2d", |b| {
        b.iter(|| triangle2d(black_box(&mut pixel_buffer)))
    });
    c.bench_function("triangle2d_rgb", |b| {
        b.iter(|| triangle2d_rgb(black_box(&mut pixel_buffer)))
    });
    let texture = TextureShader {
        texture: &std::array::from_fn::<_, 10_000, _>(|i| {
            Srgb::rgb(
                (i % 255) as u8,
                ((i + 1) % 255) as u8,
                ((i + 2) % 255) as u8,
            )
        }),
        width: 100,
        height: 100,
        sampler: rast::Sampler::Nearest,
    };
    c.bench_function("triangle2d_texture", |b| {
        b.iter(|| triangle2d_texture(black_box(&mut pixel_buffer), black_box(texture)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
