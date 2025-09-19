#![no_std]
extern crate alloc;

use core::marker::PhantomData;

use color::{LinearRgb, Srgb};
use math::{Vec2, Vec3};

pub mod color;
pub mod math;

pub mod prelude {
    pub use crate::color::{Hsv, LinearRgb, Srgb};
    pub use crate::math::{Vec2, Vec3};
    pub use crate::{ColorShader, PixelBuffer, TextureShader};
}

pub struct PixelBuffer {
    pub pixels: alloc::boxed::Box<[Srgb]>,
    pub depth_buffer: alloc::boxed::Box<[f32]>,
    pub width: usize,
    pub height: usize,
}

impl PixelBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: alloc::vec![Srgb::default(); width * height].into_boxed_slice(),
            depth_buffer: alloc::vec![0.0; width * height].into_boxed_slice(),
            width,
            height,
        }
    }
}

pub fn rast_triangle<S: Shader>(
    pixel_buffer: &mut PixelBuffer,
    v1: Vec2,
    v2: Vec2,
    v3: Vec2,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    shader: S,
) {
    rast_triangle_shaded(
        pixel_buffer,
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

pub fn rast_triangle_colored<T>(pixel_buffer: &mut PixelBuffer, v1: Vec2, v2: Vec2, v3: Vec2, c: T)
where
    T: Into<LinearRgb>,
{
    let c = c.into();
    rast_triangle_shaded(
        pixel_buffer,
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

pub fn rast_triangle_checked<S: Shader>(
    pixel_buffer: &mut PixelBuffer,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    shader: S,
) {
    rast_triangle_shaded(pixel_buffer, v1, v2, v3, d1, d2, d3, shader, true);
}

// TODO:
// - Rounding
// - Clipping
fn rast_triangle_shaded<S: Shader>(
    pixel_buffer: &mut PixelBuffer,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    d1: S::VertexData,
    d2: S::VertexData,
    d3: S::VertexData,
    mut shader: S,
    depth_check: bool,
) {
    // https://www.sunshine2k.de/coding/java/TriangleRasterization/TriangleRasterization.html
    //
    // Split the traingle into two components where one has a flat bottom and another
    // a flat top. Compute the slope of each side, iterate scanlines, then apply
    // slopes to the scanline's start and end positions. This algorithm avoids overdraw,
    // as in a barycentric driven rasterizer.
    //
    // Vertex data uses a similar approach. The barycentric gradients are precomputed and
    // accumulated with each iteration of the loop. The vertex data is interpolated
    // according to a pixel's barycentric coordinate.

    // one dimensional clip
    let min_x = (v1.x.min(v2.x).min(v3.x).max(0.0)) as usize;
    let max_x = (v1.x.max(v2.x).max(v3.x).min(pixel_buffer.width as f32)) as usize;
    let min_y = (v1.y.min(v2.y).min(v3.y).max(0.0)) as usize;
    let max_y = (v1.y.max(v2.y).max(v3.y).min(pixel_buffer.height as f32)) as usize;
    if min_y == max_y || min_x == max_x || min_y > pixel_buffer.height || min_x > pixel_buffer.width
    {
        return;
    }

    // sort vertices based on y
    let mut vs = [
        (shader.vertex(v1), d1),
        (shader.vertex(v2), d2),
        (shader.vertex(v3), d3),
    ];
    vs.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
    let (v1, d1) = vs[0];
    let (v2, d2) = vs[1];
    let (v3, d3) = vs[2];

    if v2 == v3 {
        rast_triangle_flat_bottom(
            pixel_buffer,
            v1,
            v2,
            v3,
            v1.to_vec2(),
            v2.to_vec2(),
            v3.to_vec2(),
            d1,
            d2,
            d3,
            &mut shader,
            depth_check,
        );
    } else if v1 == v2 {
        rast_triangle_flat_top(
            pixel_buffer,
            v1,
            v2,
            v3,
            v1.to_vec2(),
            v2.to_vec2(),
            v3.to_vec2(),
            d1,
            d2,
            d3,
            &mut shader,
            depth_check,
        );
    } else {
        let t = (v2.y - v1.y) / (v3.y - v1.y);
        let v4 = Vec3 {
            x: v1.x + ((v2.y - v1.y) / (v3.y - v1.y)) * (v3.x - v1.x),
            y: v2.y,
            // interpolate z along edge
            z: v1.z + t * (v3.z - v1.z),
        };
        rast_triangle_flat_bottom(
            pixel_buffer,
            v1,
            v2,
            v4,
            v1.to_vec2(),
            v2.to_vec2(),
            v3.to_vec2(),
            d1,
            d2,
            d3,
            &mut shader,
            depth_check,
        );
        rast_triangle_flat_top(
            pixel_buffer,
            v2,
            v4,
            v3,
            v1.to_vec2(),
            v2.to_vec2(),
            v3.to_vec2(),
            d1,
            d2,
            d3,
            &mut shader,
            depth_check,
        );
    }

    fn rast_triangle_flat_bottom<S: Shader>(
        pixel_buffer: &mut PixelBuffer,
        v1: Vec3,
        mut v2: Vec3,
        mut v3: Vec3,
        bc1: Vec2,
        mut bc2: Vec2,
        mut bc3: Vec2,
        d1: S::VertexData,
        mut d2: S::VertexData,
        mut d3: S::VertexData,
        shader: &mut S,
        depth_check: bool,
    ) {
        if v3.x < v2.x {
            core::mem::swap(&mut v3, &mut v2);
            core::mem::swap(&mut bc3, &mut bc2);
            core::mem::swap(&mut d3, &mut d2);
        }

        let mx2 = (v2.x - v1.x) / (v2.y - v1.y);
        let mx3 = (v3.x - v1.x) / (v3.y - v1.y);
        let x2 = v1.x;
        let x3 = v1.x;

        let mz2 = (v2.z - v1.z) / (v2.y - v1.y);
        let mz3 = (v3.z - v1.z) / (v3.y - v1.y);
        let z2 = v1.z;
        let z3 = v1.z;

        rast_triangle_inner(
            pixel_buffer,
            v1.y,
            v2.y,
            //
            x2,
            x3,
            mx2,
            mx3,
            //
            z2,
            z3,
            mz2,
            mz3,
            //
            bc1,
            bc2,
            bc3,
            //
            d1,
            d2,
            d3,
            shader,
            depth_check,
        );
    }

    fn rast_triangle_flat_top<S: Shader>(
        pixel_buffer: &mut PixelBuffer,
        mut v1: Vec3,
        mut v2: Vec3,
        v3: Vec3,
        mut bc1: Vec2,
        mut bc2: Vec2,
        bc3: Vec2,
        mut d1: S::VertexData,
        mut d2: S::VertexData,
        d3: S::VertexData,
        shader: &mut S,
        depth_check: bool,
    ) {
        if v2.x < v1.x {
            core::mem::swap(&mut v2, &mut v1);
            core::mem::swap(&mut bc2, &mut bc1);
            core::mem::swap(&mut d2, &mut d1);
        }

        let mx1 = (v1.x - v3.x) / (v1.y - v3.y);
        let mx2 = (v2.x - v3.x) / (v2.y - v3.y);
        let x1 = v1.x;
        let x2 = v2.x;

        let mz1 = (v1.z - v3.z) / (v1.y - v3.y);
        let mz2 = (v2.z - v3.z) / (v2.y - v3.y);
        let z1 = v1.z;
        let z2 = v2.z;

        rast_triangle_inner(
            pixel_buffer,
            v1.y,
            v3.y,
            //
            x1,
            x2,
            mx1,
            mx2,
            //
            z1,
            z2,
            mz1,
            mz2,
            //
            bc1,
            bc2,
            bc3,
            d1,
            d2,
            d3,
            shader,
            depth_check,
        );
    }

    fn rast_triangle_inner<S: Shader>(
        pixel_buffer: &mut PixelBuffer,
        y_start: f32,
        y_end: f32,
        //
        mut x_start: f32,
        mut x_end: f32,
        x_start_slope: f32,
        x_end_slope: f32,
        //
        mut z_start: f32,
        mut z_end: f32,
        z_start_slope: f32,
        z_end_slope: f32,
        //
        bc1: Vec2,
        bc2: Vec2,
        bc3: Vec2,
        d1: S::VertexData,
        d2: S::VertexData,
        d3: S::VertexData,
        shader: &mut S,
        depth_check: bool,
    ) {
        let (bcu_d, bcv_d, bcw_d) = barycentric_gradients(bc1, bc2, bc3);
        let mut uvw = barycentric_coordinates(Vec2::new(x_start, y_start as f32), bc1, bc2, bc3);
        let uvw_uxd = x_start_slope * bcu_d.x;
        let uvw_vxd = x_start_slope * bcv_d.x;
        let uvw_wxd = x_start_slope * bcw_d.x;

        let y_start = y_start as usize;
        let y_end = libm::roundf(y_end) as usize;

        // TODO: fuck it, barycentric driven.

        if depth_check {
            for y in y_start..y_end {
                let mut sl_uvw = uvw;
                let s = x_start as usize;
                let e = libm::roundf(x_end) as usize;
                let zinc = if e > s {
                    (z_end - z_start) / (e - s) as f32
                } else {
                    0.0
                };
                let mut z = z_start;
                for x in s..e {
                    let index = y * pixel_buffer.width + x;
                    if index >= pixel_buffer.pixels.len() {
                        continue;
                    }

                    let dp = &mut pixel_buffer.depth_buffer[index];
                    if *dp > z {
                        *dp = z;
                        let vd = (d1 * sl_uvw.x) + (d2 * sl_uvw.y) + (d3 * sl_uvw.z);
                        let color = shader.fragment(vd);
                        pixel_buffer.pixels[index] = color.srgb();
                    }

                    sl_uvw.x += bcu_d.x;
                    sl_uvw.y += bcv_d.x;
                    sl_uvw.z += bcw_d.x;
                    z += zinc;
                }
                z_start += z_start_slope;
                z_end += z_end_slope;
                x_start += x_start_slope;
                x_end += x_end_slope;
                uvw.x += bcu_d.y + uvw_uxd;
                uvw.y += bcv_d.y + uvw_vxd;
                uvw.z += bcw_d.y + uvw_wxd;
            }
        } else {
            for y in y_start..y_end {
                let mut sl_uvw = uvw;
                let s = x_start as usize;
                let e = libm::roundf(x_end) as usize;
                for x in s..e {
                    let index = y * pixel_buffer.width + x;
                    if index >= pixel_buffer.pixels.len() {
                        continue;
                    }

                    let vd = (d1 * sl_uvw.x) + (d2 * sl_uvw.y) + (d3 * sl_uvw.z);
                    let color = shader.fragment(vd);
                    pixel_buffer.pixels[index] = color.srgb();

                    sl_uvw.x += bcu_d.x;
                    sl_uvw.y += bcv_d.x;
                    sl_uvw.z += bcw_d.x;
                }
                x_start += x_start_slope;
                x_end += x_end_slope;
                uvw.x += bcu_d.y + uvw_uxd;
                uvw.y += bcv_d.y + uvw_vxd;
                uvw.z += bcw_d.y + uvw_wxd;
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

pub fn barycentric_gradients(v1: Vec2, v2: Vec2, v3: Vec2) -> (Vec2, Vec2, Vec2) {
    let d = (v1 - v3).cross(v2 - v3);
    let inv_d = 1.0 / d;
    let ug = Vec2::new(
        // ∂u/∂x
        (v2.y - v3.y) * inv_d,
        // ∂u/∂y
        (v3.x - v2.x) * inv_d,
    );
    let vg = Vec2::new(
        // ∂v/∂x
        (v3.y - v1.y) * inv_d,
        // ∂v/∂y
        (v1.x - v3.x) * inv_d,
    );
    let wg = -Vec2::new(ug.x + vg.x, ug.y + vg.y);
    (ug, vg, wg)
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
    Linear,
    Bilinear,
}

impl<T> Shader for TextureShader<'_, T>
where
    T: Copy + Into<LinearRgb>,
{
    type VertexData = Vec2;

    fn fragment(&mut self, data: Self::VertexData) -> LinearRgb {
        match self.sampler {
            Sampler::Nearest => {
                let x = (data.x * self.width as f32 + 0.5) as usize;
                let y = (data.y * self.height as f32 + 0.5) as usize;
                let len = self.texture.len().saturating_sub(1);
                self.texture[(y * self.height + x).clamp(0, len)].into()
            }
            Sampler::Linear => {
                todo!()
            }
            Sampler::Bilinear => {
                todo!()
            }
        }
    }
}

pub mod bounding_box {
    use super::*;

    pub fn rast_triangle2d_bounding_box<T>(
        pixel_buffer: &mut PixelBuffer,
        v1: Vec2,
        v2: Vec2,
        v3: Vec2,
        c: T,
    ) where
        T: Into<Srgb>,
    {
        let c = c.into();
        let min_x = (v1.x.min(v2.x).min(v3.x).max(0.0)) as usize;
        let max_x = (v1.x.max(v2.x).max(v3.x).min(pixel_buffer.width as f32)) as usize;
        let min_y = (v1.y.min(v2.y).min(v3.y).max(0.0)) as usize;
        let max_y = (v1.y.max(v2.y).max(v3.y).min(pixel_buffer.height as f32)) as usize;

        if min_y == max_y || min_x == max_x {
            return;
        }

        for y in min_y..max_y {
            for x in min_x..max_x {
                pixel_buffer.pixels[y * pixel_buffer.width + x] = c;
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
