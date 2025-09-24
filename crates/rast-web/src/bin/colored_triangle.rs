use rast::*;
use rast_web::{HEIGHT, WIDTH, serve};

fn main() {
    serve(colored_triangle);
}

fn colored_triangle(pixels: &mut [Srgb], _: &mut [f32], _: f32) {
    let v1 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 2.0);
    let v2 = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 3.0);
    let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.0);

    let c1 = LinearRgb::rgb(1.0, 0.0, 0.0);
    let c2 = LinearRgb::rgb(0.0, 1.0, 0.0);
    let c3 = LinearRgb::rgb(0.0, 0.0, 1.0);

    rast::rast_triangle(pixels, WIDTH, HEIGHT, v1, v2, v3, c1, c2, c3, ColorShader);
}
