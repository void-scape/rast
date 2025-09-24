#![no_std]
extern crate alloc;

use core::marker::PhantomData;
use tint::*;

pub mod math;

pub use math::*;
pub use tint;

pub fn rast_triangle<S: Shader, Pixel: Color>(
    pixels: &mut [Pixel],
    width: usize,
    height: usize,
    v1: Vec2,
    v2: Vec2,
    v3: Vec2,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    shader: S,
) {
    rast_triangle_shaded(
        pixels,
        &mut [],
        width,
        height,
        v1.extend(0.0),
        v2.extend(0.0),
        v3.extend(0.0),
        d1,
        d2,
        d3,
        shader,
        false,
    );
}

pub fn rast_triangle_checked<S: Shader>(
    pixels: &mut [Srgb],
    depth_buffer: &mut [f32],
    width: usize,
    height: usize,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    shader: S,
) {
    rast_triangle_shaded(
        pixels,
        depth_buffer,
        width,
        height,
        v1,
        v2,
        v3,
        d1,
        d2,
        d3,
        shader,
        true,
    );
}

pub fn rast_triangle_colored<T>(
    pixels: &mut [Srgb],
    width: usize,
    height: usize,
    v1: Vec2,
    v2: Vec2,
    v3: Vec2,
    c: T,
) where
    T: Into<LinearRgb>,
{
    let c = c.into();
    rast_triangle_shaded(
        pixels,
        &mut [],
        width,
        height,
        v1.extend(0.0),
        v2.extend(0.0),
        v3.extend(0.0),
        empty::EmptyVertexData,
        empty::EmptyVertexData,
        empty::EmptyVertexData,
        FnShader::new(|v| v, |_| c),
        false,
    );
}

fn rast_triangle_shaded<S: Shader, Pixel: Color>(
    pixels: &mut [Pixel],
    depth_buffer: &mut [f32],
    width: usize,
    height: usize,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    mut shader: S,
    depth_check: bool,
) {
    // bounding box clip
    let min_x = (v1.x.min(v2.x).min(v3.x).max(0.0)) as usize;
    let max_x = libm::ceilf(v1.x.max(v2.x).max(v3.x).min(width as f32)) as usize;
    let min_y = (v1.y.min(v2.y).min(v3.y).max(0.0)) as usize;
    let max_y = libm::ceilf(v1.y.max(v2.y).max(v3.y).min(height as f32)) as usize;
    if min_y == max_y || min_x == max_x || min_y > height || min_x > width {
        return;
    }

    // I first saw this method used here:
    //
    // https://github.com/tsoding/olive.c/blob/master/olive.c
    for y in min_y..max_y.min(height) {
        for x in min_x..max_x.min(width) {
            let index = y * width + x;
            let bc = barycentric_coordinates(
                Vec2::new(x as f32, y as f32),
                v1.to_vec2(),
                v2.to_vec2(),
                v3.to_vec2(),
            );
            if bc.x > 0.0 && bc.x < 1.0 && bc.y > 0.0 && bc.y < 1.0 && bc.z > 0.0 && bc.z < 1.0 {
                if depth_check {
                    let z = (v1.z * bc.x) + (v2.z * bc.y) + (v3.z * bc.z);
                    if depth_buffer[index] <= z {
                        continue;
                    }
                    depth_buffer[index] = z;
                }

                let vd = (d1 * bc.x) + (d2 * bc.y) + (d3 * bc.z);
                let color = shader.fragment(vd);
                pixels[index] = color.into();
            }
        }
    }
}

pub fn barycentric_coordinates(p: Vec2, v1: Vec2, v2: Vec2, v3: Vec2) -> Vec3 {
    // https://en.wikipedia.org/wiki/Barycentric_coordinate_system#Edge_approach
    let d = (v1 - v3).cross(v2 - v3);
    let u = (p - v3).cross(v2 - v3) / d;
    let v = (p - v3).cross(v3 - v1) / d;
    let w = 1.0 - u - v;
    Vec3 { x: u, y: v, z: w }
}

pub trait Shader {
    type VertexData: Copy
        + core::ops::Add<Self::VertexData, Output = Self::VertexData>
        + core::ops::Mul<f32, Output = Self::VertexData>;

    #[inline]
    fn vertex(&mut self, v: Vec3) -> Vec3 {
        v
    }

    #[inline]
    fn fragment(&mut self, data: Self::VertexData) -> LinearRgb {
        let _ = data;
        LinearRgb::rgb(1.0, 1.0, 1.0)
    }
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
    V: FnMut(Vec3) -> Vec3,
    F: FnMut(D) -> LinearRgb,
    D: Copy + core::ops::Add<D, Output = D> + core::ops::Mul<f32, Output = D>,
{
    type VertexData = D;

    fn vertex(&mut self, v: Vec3) -> Vec3 {
        self.0(v)
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
    type VertexData = Vec2;

    fn fragment(&mut self, data: Self::VertexData) -> LinearRgb {
        match self.sampler {
            Sampler::Nearest => {
                let x = libm::roundf(data.x * self.width as f32) as usize;
                let y = libm::roundf(data.y * self.height as f32) as usize;
                let len = self.texture.len().saturating_sub(1);
                self.texture[(y * self.height + x).clamp(0, len)].into()
            }
            Sampler::Bilinear => {
                // https://en.wikipedia.org/wiki/Bilinear_interpolation

                let xf = (data.x * self.width as f32).max(0.0);
                let yf = (data.y * self.height as f32).max(0.0);

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
