use rast::prelude::*;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tiny_http::{Header, Response, Server};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

struct DoubleBuffer {
    front: PixelBuffer,
    back: PixelBuffer,
}

impl DoubleBuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            front: PixelBuffer::new(width, height),
            back: PixelBuffer::new(width, height),
        }
    }

    fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }
}

fn main() {
    let double_buffer = Arc::new(RwLock::new(DoubleBuffer::new(WIDTH, HEIGHT)));
    thread::spawn({
        let double_buffer = double_buffer.clone();
        move || {
            let nlights_bytes = include_bytes!("../nlights.bin");
            let nlights = unsafe {
                core::slice::from_raw_parts(
                    nlights_bytes.as_ptr() as *const Srgb,
                    nlights_bytes.len() / 4,
                )
            }
            .to_vec();

            let mut dt = 0f32;
            loop {
                {
                    let mut buffers = double_buffer.write().unwrap();
                    let pixel_buffer = &mut buffers.back;
                    pixel_buffer.pixels.fill(Srgb::rgb(42, 42, 42));

                    colored_triangle(pixel_buffer, dt);
                    texture_quad(pixel_buffer, &nlights);

                    buffers.swap();
                }

                thread::sleep(Duration::from_millis(16));
                dt += 16.0 / 1000.0;
            }
        }
    });
    server(double_buffer.clone());
}

fn colored_triangle(pixel_buffer: &mut PixelBuffer, dt: f32) {
    let v1 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 2.0);
    let v2 = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 3.0);
    let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.0);

    let c1 = Hsv::new(dt % 1.0, 1.0, 1.0).linear();
    let c2 = Hsv::new((dt + 0.33) % 1.0, 1.0, 1.0).linear();
    let c3 = Hsv::new((dt + 0.66) % 1.0, 1.0, 1.0).linear();

    rast::rast_triangle2d_shaded(pixel_buffer, v1, v2, v3, c1, c2, c3, ColorShader);
}

fn texture_quad(pixel_buffer: &mut PixelBuffer, nlights: &[Srgb]) {
    let shader = TextureShader {
        texture: &nlights,
        width: 400,
        height: 400,
        sampler: rast::Sampler::Nearest,
    };

    let v1 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 2.2);
    let v2 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 0.8);
    let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.2);

    let uv1 = Vec2::new(0.0, 1.0);
    let uv2 = Vec2::new(0.0, 0.0);
    let uv3 = Vec2::new(1.0, 1.0);

    rast::rast_triangle2d_shaded(pixel_buffer, v1, v2, v3, uv1, uv2, uv3, shader);

    let v1 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 0.8);
    let v2 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 0.8);
    let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.2);

    let uv1 = Vec2::new(1.0, 0.0);
    let uv2 = Vec2::new(0.0, 0.0);
    let uv3 = Vec2::new(1.0, 1.0);

    rast::rast_triangle2d_shaded(pixel_buffer, v1, v2, v3, uv1, uv2, uv3, shader);
}

fn server(double_buffer: Arc<RwLock<DoubleBuffer>>) {
    let server = Server::http("localhost:3030").unwrap();
    println!("Server running at http://localhost:3030");

    for request in server.incoming_requests() {
        let html = include_str!("index.html");
        let response = if request.url() == "/pixels" {
            let buffers = double_buffer.read().unwrap();
            let pixel_data = unsafe {
                std::slice::from_raw_parts::<u8>(
                    buffers.front.pixels.as_ptr() as *const u8,
                    WIDTH * HEIGHT * 4,
                )
            };

            Response::from_data(pixel_data.to_vec()).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..]).unwrap(),
            )
        } else {
            Response::from_string(html)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap())
        };

        let _ = request.respond(response);
    }
}
