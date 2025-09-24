# Rast

*Rast* is a simple, `no_std` software *R*ust r*ast*erizer driven by `Shader` types.

## Example

```rust
fn colored_triangle(pixel_buffer: &mut rast::PixelBuffer) {
    let v1 = rast::Vec2::new(WIDTH as f32 / 3.0, HEIGHT as f32 / 3.0 * 2.0);
    let v2 = rast::Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 3.0);
    let v3 = rast::Vec2::new(WIDTH as f32 / 3.0 * 2.0, HEIGHT as f32 / 3.0 * 2.0);

    let c1 = rast::LinearRgb::rgb(1.0, 0.0, 0.0);
    let c2 = rast::LinearRgb::rgb(0.0, 1.0, 0.0);
    let c3 = rast::LinearRgb::rgb(0.0, 0.0, 1.0);

    rast::rast_triangle(pixel_buffer, v1, v2, v3, c1, c2, c3, rast::ColorShader);
}
```

## Demos

`cargo run --bin utah_teapot`<br>

<img src="assets/utah_teapot.gif" width="520">

---

`cargo run --bin texture_quad`<br>

<img src="assets/texture_quad.gif" width="520">

---

`cargo run --bin colored_triangle`<br>

<img src="assets/colored_triangle.png" width="520">
