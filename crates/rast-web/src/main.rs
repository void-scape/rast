use rast::{Color, PixelBuffer, Vec2};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tiny_http::{Header, Response, Server};

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

fn main() {
    let pixel_buffer = Arc::new(Mutex::new(PixelBuffer::new(WIDTH, HEIGHT)));
    thread::spawn({
        let pixel_buffer = pixel_buffer.clone();
        move || {
            loop {
                let mut pixel_buffer = pixel_buffer.lock().unwrap();
                {
                    let v1 = Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 2.0);
                    let v2 = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 3.0);
                    let v3 = Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.0);
                    let c1 = Color::rgb(255, 0, 0);
                    let c2 = Color::rgb(0, 255, 0);
                    let c3 = Color::rgb(0, 0, 255);

                    pixel_buffer.pixels.fill(Color::default());
                    rast::rast_triangle2d(&mut pixel_buffer, v1, v2, v3, c1, c2, c3);
                }
                thread::sleep(Duration::from_millis(16));
            }
        }
    });

    let server = Server::http("localhost:3030").unwrap();
    println!("Server running at http://localhost:3030");

    for request in server.incoming_requests() {
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head><title>rasp-web</title></head>
<body>
    <canvas id="canvas" width="{}" height="{}"></canvas>
    <script>
        const canvas = document.getElementById('canvas');
        const ctx = canvas.getContext('2d');
        function updateFrame() {{
            fetch('/pixels')
                .then(r => r.json())
                .then(data => {{
                    const imageData = ctx.createImageData({}, {});
                    imageData.data.set(new Uint8Array(data));
                    ctx.putImageData(imageData, 0, 0);
                    requestAnimationFrame(updateFrame);
                }});
        }}
        setTimeout(() => updateFrame(), 100);
    </script>
</body>
</html>
        "#,
            WIDTH, HEIGHT, WIDTH, HEIGHT
        );

        let response = if request.url() == "/pixels" {
            let pixels = pixel_buffer.lock().unwrap();
            let pixel_data = unsafe {
                std::slice::from_raw_parts::<u8>(
                    pixels.pixels.as_ptr() as *const u8,
                    WIDTH * HEIGHT * 4,
                )
            };
            Response::from_string(format!("{:?}", pixel_data)).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            )
        } else {
            Response::from_string(html)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap())
        };

        let _ = request.respond(response);
    }
}
