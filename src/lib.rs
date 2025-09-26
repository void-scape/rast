// TODO: There is either a precision issue in the zbuffer or a bug somewhere upstream
// causing z fighting.

#![no_std]
extern crate alloc;

use core::marker::PhantomData;
use tint::*;

pub use tint;

pub fn rast_triangle<S: Shader, Pixel: Color>(
    pixels: &mut [Pixel],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v2x: f32,
    v2y: f32,
    v3x: f32,
    v3y: f32,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    shader: S,
) {
    #[rustfmt::skip]
    rast_triangle_inner(
        pixels,
        &mut [],
        width,
        height,
        v1x, v1y, 0.0,
        v2x, v2y, 0.0,
        v3x, v3y, 0.0,
        d1,
        d2,
        d3,
        shader,
        false,
    );
}

pub fn rast_triangle_checked<S: Shader>(
    pixels: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v1z: f32,
    v2x: f32,
    v2y: f32,
    v2z: f32,
    v3x: f32,
    v3y: f32,
    v3z: f32,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    shader: S,
) {
    #[rustfmt::skip]
    rast_triangle_inner(
        pixels,
        zbuffer,
        width,
        height,
        v1x, v1y, v1z,
        v2x, v2y, v2z,
        v3x, v3y, v3z,
        d1,
        d2,
        d3,
        shader,
        true,
    );
}

pub fn rast_triangle_colored<Pixel: Copy>(
    pixels: &mut [Pixel],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v2x: f32,
    v2y: f32,
    v3x: f32,
    v3y: f32,
    c: Pixel,
) {
    // bounding box clip
    let minx = (v1x.min(v2x).min(v3x).max(0.0)) as usize;
    let maxx = libm::ceilf(v1x.max(v2x).max(v3x).min(width as f32)) as usize;
    let miny = (v1y.min(v2y).min(v3y).max(0.0)) as usize;
    let maxy = libm::ceilf(v1y.max(v2y).max(v3y).min(height as f32)) as usize;
    if miny == maxy || minx == maxx || miny > height || minx > width {
        return;
    }

    for y in miny..maxy.min(height) {
        for x in minx..maxx.min(width) {
            if barycentric_coordinates(x as f32, y as f32, v1x, v1y, v2x, v2y, v3x, v3y).is_some() {
                let index = y * width + x;
                pixels[index] = c;
            }
        }
    }
}

pub fn rast_line<Pixel: Copy>(
    pixels: &mut [Pixel],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v1z: f32,
    v2x: f32,
    v2y: f32,
    v2z: f32,
    c: Pixel,
) {
    #[rustfmt::skip]
    rast_line_inner(
        pixels,
        &mut [],
        width,
        height,
        v1x, v1y, v1z,
        v2x, v2y, v2z,
        c,
        false,
    );
}

pub fn rast_line_checked<Pixel: Copy>(
    pixels: &mut [Pixel],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v1z: f32,
    v2x: f32,
    v2y: f32,
    v2z: f32,
    c: Pixel,
) {
    #[rustfmt::skip]
    rast_line_inner(
        pixels,
        zbuffer,
        width,
        height,
        v1x, v1y, v1z,
        v2x, v2y, v2z,
        c,
        true,
    );
}

fn rast_line_inner<Pixel: Copy>(
    pixels: &mut [Pixel],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v1z: f32,
    v2x: f32,
    v2y: f32,
    v2z: f32,
    c: Pixel,
    depth_check: bool,
) {
    let dx = v2x - v1x;
    let dy = v2y - v1y;
    if dx.abs() < 1.0 && dy.abs() < 1.0 {
        return;
    }

    let steps = if dx.abs() > dy.abs() {
        libm::ceilf(dx.abs()) as i32
    } else {
        libm::ceilf(dy.abs()) as i32
    };

    let step_x = dx / steps as f32;
    let step_y = dy / steps as f32;
    let step_z = (v2z - v1z) / steps as f32;

    for i in 0..=steps {
        let x = v1x + i as f32 * step_x;
        let y = v1y + i as f32 * step_y;
        let z = v1z + i as f32 * step_z;

        let pixel_x = libm::floorf(x) as i32;
        let pixel_y = libm::floorf(y) as i32;

        if pixel_x >= 0 && pixel_x < width as i32 && pixel_y >= 0 && pixel_y < height as i32 {
            let index = (pixel_y as usize) * width + (pixel_x as usize);

            if depth_check {
                if zbuffer[index] <= z {
                    continue;
                }
                zbuffer[index] = z;
            }
            pixels[index] = c;
        }
    }
}

pub fn rast_triangle_wireframe<Pixel: Copy>(
    pixels: &mut [Pixel],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v1z: f32,
    v2x: f32,
    v2y: f32,
    v2z: f32,
    v3x: f32,
    v3y: f32,
    v3z: f32,
    c: Pixel,
) {
    #[rustfmt::skip]
    rast_line(
        pixels,
        width,
        height,
        v1x, v1y, v1z,
        v2x, v2y, v2z,
        c,
    );
    #[rustfmt::skip]
    rast_line(
        pixels,
        width,
        height,
        v1x, v1y, v1z,
        v3x, v3y, v3z,
        c,
    );
    #[rustfmt::skip]
    rast_line(
        pixels,
        width,
        height,
        v2x, v2y, v2z,
        v3x, v3y, v3z,
        c,
    );
}

pub fn rast_triangle_wireframe_checked<Pixel: Copy>(
    pixels: &mut [Pixel],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v1z: f32,
    v2x: f32,
    v2y: f32,
    v2z: f32,
    v3x: f32,
    v3y: f32,
    v3z: f32,
    c: Pixel,
) {
    #[rustfmt::skip]
    rast_line_checked(
        pixels,
        zbuffer,
        width,
        height,
        v1x, v1y, v1z,
        v2x, v2y, v2z,
        c,
    );
    #[rustfmt::skip]
    rast_line_checked(
        pixels,
        zbuffer,
        width,
        height,
        v1x, v1y, v1z,
        v3x, v3y, v3z,
        c,
    );
    #[rustfmt::skip]
    rast_line_checked(
        pixels,
        zbuffer,
        width,
        height,
        v2x, v2y, v2z,
        v3x, v3y, v3z,
        c,
    );
}

fn rast_triangle_inner<S: Shader, Pixel: Color>(
    pixels: &mut [Pixel],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    v1x: f32,
    v1y: f32,
    v1z: f32,
    v2x: f32,
    v2y: f32,
    v2z: f32,
    v3x: f32,
    v3y: f32,
    v3z: f32,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    mut shader: S,
    depth_check: bool,
) {
    // bounding box clip
    let minx = (v1x.min(v2x).min(v3x).max(0.0)) as usize;
    let maxx = libm::ceilf(v1x.max(v2x).max(v3x).min(width as f32)) as usize;
    let miny = (v1y.min(v2y).min(v3y).max(0.0)) as usize;
    let maxy = libm::ceilf(v1y.max(v2y).max(v3y).min(height as f32)) as usize;
    if miny == maxy || minx == maxx || miny > height || minx > width {
        return;
    }

    let (v1x, v1y, v1z) = shader.vertex(v1x, v1y, v1z);
    let (v2x, v2y, v2z) = shader.vertex(v2x, v2y, v2z);
    let (v3x, v3y, v3z) = shader.vertex(v3x, v3y, v3z);

    // I first saw this method used here:
    //
    // https://github.com/tsoding/olive.c/blob/master/olive.c
    for y in miny..maxy.min(height) {
        for x in minx..maxx.min(width) {
            let index = y * width + x;
            if let Some((bcx, bcy, bcz)) =
                barycentric_coordinates(x as f32, y as f32, v1x, v1y, v2x, v2y, v3x, v3y)
            {
                if depth_check {
                    let z = (v1z * bcx) + (v2z * bcy) + (v3z * bcz);
                    if zbuffer[index] <= z {
                        continue;
                    }
                    zbuffer[index] = z;
                }
                let vd = shader.interpolate(bcx, bcy, bcz, d1, d2, d3);
                let color = shader.fragment(vd);
                pixels[index] = color.into();
            }
        }
    }
}

pub fn barycentric_coordinates(
    px: f32,
    py: f32,
    v1x: f32,
    v1y: f32,
    v2x: f32,
    v2y: f32,
    v3x: f32,
    v3y: f32,
) -> Option<(f32, f32, f32)> {
    // https://en.wikipedia.org/wiki/Barycentric_coordinate_system#Edge_approach
    let d = (v1x - v3x) * (v2y - v3y) - (v1y - v3y) * (v2x - v3x);
    let u = ((px - v3x) * (v2y - v3y) - (py - v3y) * (v2x - v3x)) / d;
    let v = ((px - v3x) * (v3y - v1y) - (py - v3y) * (v3x - v1x)) / d;
    let w = 1.0 - u - v;

    if u > 0.0 && u < 1.0 && v > 0.0 && v < 1.0 && w > 0.0 && w < 1.0 {
        Some((u, v, w))
    } else {
        None
    }
}

pub trait Shader {
    type VertexData: Copy;

    fn interpolate(
        &self,
        bcx: f32,
        bcy: f32,
        bcz: f32,
        d1: Self::VertexData,
        d2: Self::VertexData,
        d3: Self::VertexData,
    ) -> Self::VertexData;

    #[inline]
    fn vertex(&mut self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        (x, y, z)
    }

    #[inline]
    fn fragment(&mut self, data: Self::VertexData) -> LinearRgb {
        let _ = data;
        LinearRgb::rgb(1.0, 1.0, 1.0)
    }
}

pub fn barycentric_lerp<T>(bcx: f32, bcy: f32, bcz: f32, d1: T, d2: T, d3: T) -> T
where
    T: core::ops::Add<T, Output = T> + core::ops::Mul<f32, Output = T>,
{
    (d1 * bcx) + (d2 * bcy) + (d3 * bcz)
}

#[derive(Debug, Clone, Copy)]
pub struct FnShader<V, F, D>(V, F, PhantomData<D>);

impl<V, F, D> FnShader<V, F, D> {
    pub fn new(vertex: V, fragment: F) -> Self {
        Self(vertex, fragment, PhantomData)
    }
}

impl<V, F, D> Shader for FnShader<V, F, D>
where
    V: FnMut(f32, f32, f32) -> (f32, f32, f32),
    F: FnMut(D) -> LinearRgb,
    D: Copy + core::ops::Add<D, Output = D> + core::ops::Mul<f32, Output = D>,
{
    type VertexData = D;

    #[inline]
    fn interpolate(
        &self,
        bcx: f32,
        bcy: f32,
        bcz: f32,
        d1: Self::VertexData,
        d2: Self::VertexData,
        d3: Self::VertexData,
    ) -> Self::VertexData {
        barycentric_lerp(bcx, bcy, bcz, d1, d2, d3)
    }

    #[inline]
    fn vertex(&mut self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        self.0(x, y, z)
    }

    #[inline]
    fn fragment(&mut self, data: Self::VertexData) -> LinearRgb {
        self.1(data)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorShader;

impl Shader for ColorShader {
    type VertexData = LinearRgb;

    #[inline]
    fn interpolate(
        &self,
        bcx: f32,
        bcy: f32,
        bcz: f32,
        d1: Self::VertexData,
        d2: Self::VertexData,
        d3: Self::VertexData,
    ) -> Self::VertexData {
        barycentric_lerp(bcx, bcy, bcz, d1, d2, d3)
    }

    #[inline]
    fn fragment(&mut self, data: Self::VertexData) -> LinearRgb {
        data
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureShader<'a, T> {
    pub texture: &'a [T],
    pub width: usize,
    pub height: usize,
    pub sampler: Sampler,
}

#[derive(Debug, Clone, Copy)]
pub enum Sampler {
    Nearest,
    Bilinear,
}

impl<T> Shader for TextureShader<'_, T>
where
    T: Copy + Color,
{
    type VertexData = (f32, f32);

    fn interpolate(
        &self,
        bcx: f32,
        bcy: f32,
        bcz: f32,
        d1: Self::VertexData,
        d2: Self::VertexData,
        d3: Self::VertexData,
    ) -> Self::VertexData {
        let u = barycentric_lerp(bcx, bcy, bcz, d1.0, d2.0, d3.0);
        let v = barycentric_lerp(bcx, bcy, bcz, d1.1, d2.1, d3.1);
        (u, v)
    }

    fn fragment(&mut self, data: Self::VertexData) -> LinearRgb {
        let (u, v) = data;
        match self.sampler {
            Sampler::Nearest => {
                let x = libm::roundf(u * self.width as f32) as usize;
                let y = libm::roundf(v * self.height as f32) as usize;
                let len = self.texture.len().saturating_sub(1);
                self.texture[(y * self.height + x).clamp(0, len)].into()
            }
            Sampler::Bilinear => {
                // https://en.wikipedia.org/wiki/Bilinear_interpolation

                let xf = (u * self.width as f32).max(0.0);
                let yf = (v * self.height as f32).max(0.0);

                let x0 = (xf as usize).min(self.width - 1);
                let x1 = (x0 + 1).min(self.width - 1);
                let y0 = (yf as usize).min(self.height - 1);
                let y1 = (y0 + 1).min(self.height - 1);

                let dx = xf - x0 as f32;
                let dy = yf - y0 as f32;

                let c00: LinearRgb = self.texture[y0 * self.width + x0].into();
                let c10: LinearRgb = self.texture[y0 * self.width + x1].into();
                let c01: LinearRgb = self.texture[y1 * self.width + x0].into();
                let c11: LinearRgb = self.texture[y1 * self.width + x1].into();

                let top = c00 * (1.0 - dx) + c10 * dx;
                let bottom = c01 * (1.0 - dx) + c11 * dx;
                top * (1.0 - dy) + bottom * dy
            }
        }
    }
}

pub mod empty {
    use crate::Shader;

    pub struct EmptyShader;
    impl Shader for EmptyShader {
        type VertexData = EmptyVertexData;
        fn interpolate(
            &self,
            _bcx: f32,
            _bcy: f32,
            _bcz: f32,
            _d1: Self::VertexData,
            _d2: Self::VertexData,
            _d3: Self::VertexData,
        ) -> Self::VertexData {
            EmptyVertexData
        }
    }
    #[derive(Clone, Copy)]
    pub struct EmptyVertexData;
    impl core::ops::Add for EmptyVertexData {
        type Output = Self;
        fn add(self, _: Self) -> Self::Output {
            EmptyVertexData
        }
    }
    impl core::ops::Mul<f32> for EmptyVertexData {
        type Output = Self;
        fn mul(self, _: f32) -> Self::Output {
            EmptyVertexData
        }
    }
}
