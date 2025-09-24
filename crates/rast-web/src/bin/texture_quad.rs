use rast::tint::*;
use rast::*;
use rast_web::{HEIGHT, WIDTH, serve};

fn main() {
    let nlight_bytes = include_bytes!("../../assets/nlights.bin");
    let nlights = unsafe {
        std::slice::from_raw_parts(nlight_bytes.as_ptr() as *const Srgb, nlight_bytes.len() / 4)
    };
    let shader = TextureShader {
        texture: nlights,
        width: 400,
        height: 400,
        sampler: rast::Sampler::Bilinear,
    };

    let mut angle = 0.0;
    serve(move |pixel_buffer, _, dt| {
        angle += dt;

        let scale = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0);
        let offset = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);

        let v1 = transform_vertex(Vec3::new(-0.5, 0.7, 0.0), angle, scale, offset);
        let v2 = transform_vertex(Vec3::new(-0.5, -0.7, 0.0), angle, scale, offset);
        let v3 = transform_vertex(Vec3::new(0.5, 0.7, 0.0), angle, scale, offset);
        let uv1 = Vec2::new(0.0, 1.0);
        let uv2 = Vec2::new(0.0, 0.0);
        let uv3 = Vec2::new(1.0, 1.0);
        rast::rast_triangle(
            pixel_buffer,
            WIDTH,
            HEIGHT,
            v1,
            v2,
            v3,
            uv1,
            uv2,
            uv3,
            shader,
        );

        let v1 = transform_vertex(Vec3::new(0.5, -0.7, 0.0), angle, scale, offset);
        let v2 = transform_vertex(Vec3::new(-0.5, -0.7, 0.0), angle, scale, offset);
        let v3 = transform_vertex(Vec3::new(0.5, 0.7, 0.0), angle, scale, offset);
        let uv1 = Vec2::new(1.0, 0.0);
        let uv2 = Vec2::new(0.0, 0.0);
        let uv3 = Vec2::new(1.0, 1.0);
        rast::rast_triangle(
            pixel_buffer,
            WIDTH,
            HEIGHT,
            v1,
            v2,
            v3,
            uv1,
            uv2,
            uv3,
            shader,
        );
    });
}

fn transform_vertex(v: Vec3, angle: f32, scale: Vec2, offset: Vec2) -> Vec2 {
    let rotated = v.rotate_z(angle);
    Vec2::new(
        rotated.x * scale.x + offset.x,
        rotated.y * scale.y + offset.y,
    )
}
