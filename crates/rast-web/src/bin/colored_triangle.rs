use rast::tint::*;
use rast::*;
use rast_web::{HEIGHT, WIDTH, serve};

fn main() {
    serve(colored_triangle);
}

fn colored_triangle(pixels: &mut [Srgb], _: &mut [f32], _: f32) {
    rast::rast_triangle(
        pixels,
        WIDTH,
        HEIGHT,
        WIDTH as f32 / 3.0,
        HEIGHT as f32 / 3.0 * 2.0,
        WIDTH as f32 / 2.0,
        HEIGHT as f32 / 3.0,
        WIDTH as f32 / 3.0 * 2.0,
        HEIGHT as f32 / 3.0 * 2.0,
        LinearRgb::rgb(1.0, 0.0, 0.0),
        LinearRgb::rgb(0.0, 1.0, 0.0),
        LinearRgb::rgb(0.0, 0.0, 1.0),
        ColorShader,
    );
}
