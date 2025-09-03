#![no_std]
extern crate alloc;

pub struct PixelBuffer {
    pub pixels: alloc::boxed::Box<[Color]>,
    pub width: usize,
    pub height: usize,
}

impl PixelBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: alloc::vec![Color::default(); width * height].into_boxed_slice(),
            width,
            height,
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

// https://www.sunshine2k.de/coding/java/TriangleRasterization/TriangleRasterization.html
// TODO: Rounding and gamma correction.
pub fn rast_triangle2d(
    pixel_buffer: &mut PixelBuffer,
    v1: Vec2,
    v2: Vec2,
    v3: Vec2,
    c1: Color,
    c2: Color,
    c3: Color,
) {
    let mut vs = [(v1, c1), (v2, c2), (v3, c3)];
    vs.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
    let (v1, c1) = vs[0];
    let (v2, c2) = vs[1];
    let (v3, c3) = vs[2];

    // TODO: These branches will almost never run, are they worth it?
    if v2 == v3 {
        rast_triangle2d_flat_bottom(pixel_buffer, v1, v2, v3, v1, v2, v3, c1, c2, c3);
    } else if v1 == v2 {
        rast_triangle2d_flat_top(pixel_buffer, v1, v2, v3, v1, v2, v3, c1, c2, c3);
    } else {
        let v4 = Vec2 {
            x: v1.x + ((v2.y - v1.y) / (v3.y - v1.y)) * (v3.x - v1.x),
            y: v2.y,
        };
        rast_triangle2d_flat_bottom(pixel_buffer, v1, v2, v4, v1, v2, v3, c1, c2, c3);
        rast_triangle2d_flat_top(pixel_buffer, v2, v4, v3, v1, v2, v3, c1, c2, c3);
    }
}

fn rast_triangle2d_flat_bottom(
    pixel_buffer: &mut PixelBuffer,
    v1: Vec2,
    mut v2: Vec2,
    mut v3: Vec2,
    bc1: Vec2,
    bc2: Vec2,
    bc3: Vec2,
    c1: Color,
    c2: Color,
    c3: Color,
) {
    if v3.x < v2.x {
        core::mem::swap(&mut v3, &mut v2);
    }

    let m2 = (v2.x - v1.x) / (v2.y - v1.y);
    let m3 = (v3.x - v1.x) / (v3.y - v1.y);

    let mut x2 = v1.x;
    let mut x3 = v1.x;

    rast_triangle2d_inner(
        pixel_buffer,
        v1.y as usize..v2.y as usize,
        &mut || {
            let range = x2 as usize..x3 as usize;
            x2 += m2;
            x3 += m3;
            range
        },
        bc1,
        bc2,
        bc3,
        c1,
        c2,
        c3,
    );
}

fn rast_triangle2d_flat_top(
    pixel_buffer: &mut PixelBuffer,
    mut v1: Vec2,
    mut v2: Vec2,
    v3: Vec2,
    bc1: Vec2,
    bc2: Vec2,
    bc3: Vec2,
    c1: Color,
    c2: Color,
    c3: Color,
) {
    if v2.x < v1.x {
        core::mem::swap(&mut v2, &mut v1);
    }

    let m1 = (v1.x - v3.x) / (v1.y - v3.y);
    let m2 = (v2.x - v3.x) / (v2.y - v3.y);

    let mut x1 = v1.x;
    let mut x2 = v2.x;

    rast_triangle2d_inner(
        pixel_buffer,
        v1.y as usize..v3.y as usize,
        &mut || {
            let range = x1 as usize..x2 as usize;
            x1 += m1;
            x2 += m2;
            range
        },
        bc1,
        bc2,
        bc3,
        c1,
        c2,
        c3,
    );
}

fn rast_triangle2d_inner(
    pixel_buffer: &mut PixelBuffer,
    y_range: core::ops::Range<usize>,
    x_range: &mut impl FnMut() -> core::ops::Range<usize>,
    bc1: Vec2,
    bc2: Vec2,
    bc3: Vec2,
    c1: Color,
    c2: Color,
    c3: Color,
) {
    let c1r = c1.r as f32;
    let c1g = c1.g as f32;
    let c1b = c1.b as f32;
    let c1a = c1.a as f32;

    let c2r = c2.r as f32;
    let c2g = c2.g as f32;
    let c2b = c2.b as f32;
    let c2a = c2.a as f32;

    let c3r = c3.r as f32;
    let c3g = c3.g as f32;
    let c3b = c3.b as f32;
    let c3a = c3.a as f32;

    for y in y_range {
        for x in x_range() {
            let p = Vec2 {
                x: x as f32,
                y: y as f32,
            };
            let uvw = barycentric_coordinates(p, bc1, bc2, bc3);
            // TODO: More efficient way to handle color.
            pixel_buffer.pixels[y * pixel_buffer.width + x] = Color {
                r: (c1r * uvw.x + c2r * uvw.y + c3r * uvw.z) as u8,
                g: (c1g * uvw.x + c2g * uvw.y + c3g * uvw.z) as u8,
                b: (c1b * uvw.x + c2b * uvw.y + c3b * uvw.z) as u8,
                a: (c1a * uvw.x + c2a * uvw.y + c3a * uvw.z) as u8,
            };
        }
    }
}

pub fn barycentric_coordinates(p: Vec2, v1: Vec2, v2: Vec2, v3: Vec2) -> Vec3 {
    let uvw = triangle2d_area(v1, v2, v3);
    let u = v1;
    let v = v2;
    let w = v3;

    Vec3 {
        x: triangle2d_area(v, w, p) / uvw,
        y: triangle2d_area(u, w, p) / uvw,
        z: triangle2d_area(u, v, p) / uvw,
    }
}

pub fn triangle2d_area(v1: Vec2, v2: Vec2, v3: Vec2) -> f32 {
    0.5 * (v1.x * (v2.y - v3.y) + v2.x * (v3.y - v1.y) + v3.x * (v1.y - v2.y)).abs()
}
