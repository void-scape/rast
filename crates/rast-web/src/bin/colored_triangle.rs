use rast::prelude::*;
use rast_web::{HEIGHT, WIDTH, serve};

fn main() {
    serve(colored_triangle);
}

fn colored_triangle(pixel_buffer: &mut PixelBuffer, dt: f32) {
    let v1 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 2.0);
    let v2 = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 3.0);
    let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.0);

    let c1 = Hsv::new(dt % 1.0, 1.0, 1.0).linear();
    let c2 = Hsv::new((dt + 0.33) % 1.0, 1.0, 1.0).linear();
    let c3 = Hsv::new((dt + 0.66) % 1.0, 1.0, 1.0).linear();

    rast::rast_triangle(pixel_buffer, v1, v2, v3, c1, c2, c3, ColorShader);
}
