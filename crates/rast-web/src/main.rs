#![allow(unused)]

use rast::prelude::*;
use std::io::BufReader;
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
    let render_time = Arc::new(RwLock::new(0.0));
    let double_buffer = Arc::new(RwLock::new(DoubleBuffer::new(WIDTH, HEIGHT)));
    thread::spawn({
        let render_time = render_time.clone();
        let double_buffer = double_buffer.clone();
        move || {
            let nlights_bytes = include_bytes!("../assets/nlights.bin");
            let nlights = unsafe {
                core::slice::from_raw_parts(
                    nlights_bytes.as_ptr() as *const Srgb,
                    nlights_bytes.len() / 4,
                )
            }
            .to_vec();

            let utah_teapot_verts = read_utah_teapot();
            let angle = &mut 0.0;

            let dt = 16.0 / 1000.0;
            loop {
                let start = std::time::Instant::now();
                {
                    let mut buffers = double_buffer.write().unwrap();
                    let pixel_buffer = &mut buffers.back;
                    pixel_buffer.pixels.fill(Srgb::rgb(42, 42, 42));
                    pixel_buffer.depth_buffer.fill(std::f32::MAX);

                    utah_teapot(pixel_buffer, &utah_teapot_verts, dt, angle);
                    // colored_triangle(pixel_buffer, dt);
                    // texture_quad(pixel_buffer, &nlights);

                    buffers.swap();
                }
                let end = std::time::Instant::now()
                    .duration_since(start)
                    .as_secs_f32();
                *render_time.write().unwrap() = end;

                thread::sleep(Duration::from_millis(16));
            }
        }
    });
    server(double_buffer.clone(), render_time.clone());
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

    rast::rast_triangle(pixel_buffer, v1, v2, v3, uv1, uv2, uv3, shader);

    let v1 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 0.8);
    let v2 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 0.8);
    let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.2);

    let uv1 = Vec2::new(1.0, 0.0);
    let uv2 = Vec2::new(0.0, 0.0);
    let uv3 = Vec2::new(1.0, 1.0);

    rast::rast_triangle(pixel_buffer, v1, v2, v3, uv1, uv2, uv3, shader);
}

fn read_utah_teapot() -> Vec<Vec3> {
    let (model, _) = tobj::load_obj_buf(
        &mut BufReader::new(include_bytes!("../assets/utah-teapot.obj").as_slice()),
        &tobj::GPU_LOAD_OPTIONS,
        |_| tobj::MTLLoadResult::Ok(Default::default()),
    )
    .unwrap();

    let mut output = Vec::new();
    for i in model[0].mesh.indices.iter() {
        let i = *i as usize;
        let pos = Vec3::new(
            model[0].mesh.positions[i * 3],
            model[0].mesh.positions[i * 3 + 1],
            model[0].mesh.positions[i * 3 + 2],
        );
        output.push(pos);
    }

    output
}

fn utah_teapot(pixel_buffer: &mut PixelBuffer, utah_teapot: &[Vec3], dt: f32, angle: &mut f32) {
    fn display(mut v: Vec3, angle: f32) -> Vec3 {
        if v.z < 0.0 {
            v.z = -v.z;
        }

        if v.z < f32::EPSILON {
            v.z += f32::EPSILON;
        }

        let proj = Vec2::new(v.x / v.z, v.y / v.z);
        Vec3::new(
            (proj.x + 1.0) / 2.0 * WIDTH as f32,
            (1.0 - (proj.y + 1.0) / 2.0) * HEIGHT as f32,
            v.z,
        )
    }

    let offset = Vec3::new(0.0, -1.5, 4.5);
    *angle = (*angle + dt) % core::f32::consts::TAU;
    for slice in utah_teapot.chunks(3) {
        rast::rast_triangle_checked(
            pixel_buffer,
            display(slice[0].rotate_y(*angle) + offset, *angle),
            display(slice[1].rotate_y(*angle) + offset, *angle),
            display(slice[2].rotate_y(*angle) + offset, *angle),
            LinearRgb::rgb(1.0, 0.0, 0.0),
            LinearRgb::rgb(0.0, 1.0, 0.0),
            LinearRgb::rgb(0.0, 0.0, 1.0),
            ColorShader,
        );
    }
}

fn server(double_buffer: Arc<RwLock<DoubleBuffer>>, render_time: Arc<RwLock<f32>>) {
    let server = Server::http("localhost:3030").unwrap();
    println!("Server running at http://localhost:3030");

    for request in server.incoming_requests() {
        let html = include_str!("index.html");
        let response = match request.url() {
            "/pixels" => {
                let buffers = double_buffer.read().unwrap();
                let pixel_data = unsafe {
                    std::slice::from_raw_parts::<u8>(
                        buffers.front.pixels.as_ptr() as *const u8,
                        WIDTH * HEIGHT * 4,
                    )
                };

                Response::from_data(pixel_data.to_vec()).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..])
                        .unwrap(),
                )
            }
            "/time" => {
                let time = render_time.read().unwrap();
                Response::from_string(format!("{time:.4}")).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap(),
                )
            }
            _ => Response::from_string(html)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap()),
        };

        let _ = request.respond(response);
    }
}
