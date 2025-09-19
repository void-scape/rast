use criterion::{Criterion, criterion_group, criterion_main};
use rast::prelude::*;
use std::hint::black_box;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn verts() -> (Vec3, Vec3, Vec3) {
    let v1 = Vec3::new(
        black_box(WIDTH as f32 / 3.0),
        black_box(HEIGHT as f32 / 3.0 * 2.0),
        0.0,
    );
    let v2 = Vec3::new(
        black_box(WIDTH as f32 / 2.0),
        black_box(HEIGHT as f32 / 3.0),
        0.0,
    );
    let v3 = Vec3::new(
        black_box(WIDTH as f32 / 3.0 * 2.0),
        black_box(HEIGHT as f32 / 3.0 * 2.0),
        0.0,
    );
    (v1, v2, v3)
}

fn triangle(pixel_buffer: &mut PixelBuffer) {
    let (v1, v2, v3) = verts();
    rast::rast_triangle_colored(
        black_box(pixel_buffer),
        black_box(v1.to_vec2()),
        black_box(v2.to_vec2()),
        black_box(v3.to_vec2()),
        Srgb::rgb(255, 255, 255),
    );
}

fn triangle_rgb(pixel_buffer: &mut PixelBuffer) {
    let (v1, v2, v3) = verts();

    let c1 = LinearRgb::rgb(black_box(1.0), black_box(0.0), black_box(0.0));
    let c2 = LinearRgb::rgb(black_box(0.0), black_box(1.0), black_box(0.0));
    let c3 = LinearRgb::rgb(black_box(0.0), black_box(0.0), black_box(1.0));

    rast::rast_triangle(
        black_box(pixel_buffer),
        black_box(v1.to_vec2()),
        black_box(v2.to_vec2()),
        black_box(v3.to_vec2()),
        black_box(c1),
        black_box(c2),
        black_box(c3),
        black_box(ColorShader),
    );
}

fn triangle_texture(pixel_buffer: &mut PixelBuffer, texture: TextureShader<Srgb>) {
    let (v1, v2, v3) = verts();

    let uv1 = Vec2::new(black_box(0.0), black_box(1.0));
    let uv2 = Vec2::new(black_box(0.0), black_box(0.0));
    let uv3 = Vec2::new(black_box(1.0), black_box(1.0));

    rast::rast_triangle(
        black_box(pixel_buffer),
        black_box(v1.to_vec2()),
        black_box(v2.to_vec2()),
        black_box(v3.to_vec2()),
        black_box(uv1),
        black_box(uv2),
        black_box(uv3),
        black_box(texture),
    );
}

fn triangle_rgb_checked(pixel_buffer: &mut PixelBuffer) {
    let (v1, v2, v3) = verts();

    let c1 = LinearRgb::rgb(black_box(1.0), black_box(0.0), black_box(0.0));
    let c2 = LinearRgb::rgb(black_box(0.0), black_box(1.0), black_box(0.0));
    let c3 = LinearRgb::rgb(black_box(0.0), black_box(0.0), black_box(1.0));

    rast::rast_triangle_checked(
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

fn bench_fn(c: &mut Criterion, name: &str, f: impl Fn(&mut PixelBuffer)) {
    c.bench_function(name, |b| {
        b.iter_batched(
            || PixelBuffer::new(WIDTH, HEIGHT),
            |mut buf| f(black_box(&mut buf)),
            criterion::BatchSize::LargeInput,
        );
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_fn(c, "triangle", triangle);
    bench_fn(c, "triangle_rgb", triangle_rgb);
    bench_fn(c, "triangle_rgb_checked", triangle_rgb_checked);

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
    c.bench_function("triangle_texture", |b| {
        b.iter_batched(
            || {
                let mut pb = PixelBuffer::new(WIDTH, HEIGHT);
                pb.depth_buffer.fill(1.0);
                pb
            },
            |mut buf| triangle_texture(black_box(&mut buf), black_box(texture)),
            criterion::BatchSize::LargeInput,
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
