use rast::prelude::*;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tiny_http::{Header, Response, Server};

pub fn serve(mut f: impl FnMut(&mut PixelBuffer, f32) + Send + Sync + 'static) {
    let render_time = Arc::new(RwLock::new(0.0));
    let double_buffer = Arc::new(RwLock::new(DoubleBuffer::new(WIDTH, HEIGHT)));
    thread::spawn({
        let render_time = render_time.clone();
        let double_buffer = double_buffer.clone();
        move || {
            let mut dt = 16.0 / 1000.0;
            loop {
                let start = std::time::Instant::now();
                {
                    let mut buffers = double_buffer.write().unwrap();
                    let pixel_buffer = &mut buffers.back;
                    pixel_buffer.pixels.fill(Srgb::rgb(42, 42, 42));
                    pixel_buffer.depth_buffer.fill(std::f32::MAX);
                    f(pixel_buffer, dt);
                    buffers.swap();
                }
                let end = std::time::Instant::now()
                    .duration_since(start)
                    .as_secs_f32();
                *render_time.write().unwrap() = end;
                dt = end;

                // Sleep so that server can read buffer.
                thread::sleep(Duration::from_millis(1));
            }
        }
    });
    server(double_buffer.clone(), render_time.clone());
}

pub const WIDTH: usize = 600;
pub const HEIGHT: usize = 600;

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
