use rast::prelude::*;
use rast_web::{HEIGHT, WIDTH, serve};

fn main() {
    let nlight_bytes = include_bytes!("../../assets/nlights.bin");
    let nlights = unsafe {
        std::slice::from_raw_parts(nlight_bytes.as_ptr() as *const Srgb, nlight_bytes.len() / 4)
    };

    serve(move |pixel_buffer, _| {
        let shader = TextureShader {
            texture: nlights,
            width: 400,
            height: 400,
            sampler: rast::Sampler::Bilinear,
        };

        let v1 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 2.2);
        let v2 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 0.8);
        let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.2);

        let uv1 = Vec2::new(0.0, 1.0);
        let uv2 = Vec2::new(0.0, 0.0);
        let uv3 = Vec2::new(1.0, 1.0);

        rast::rast_triangle(pixel_buffer, v1, v2, v3, uv1, uv2, uv3, shader);

        let v1 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 0.8);
        let v2 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 0.8);
        let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.2);

        let uv1 = Vec2::new(1.0, 0.0);
        let uv2 = Vec2::new(0.0, 0.0);
        let uv3 = Vec2::new(1.0, 1.0);

        rast::rast_triangle(pixel_buffer, v1, v2, v3, uv1, uv2, uv3, shader);
    });
}
