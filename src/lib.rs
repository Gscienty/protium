#[allow(dead_code)]
mod renderer;
mod utils;

extern crate console_error_panic_hook;
use std::panic;

#[wasm_bindgen_futures::wasm_bindgen::prelude::wasm_bindgen]
pub async fn run(canvas: web_sys::HtmlCanvasElement) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // renderer::test_draw(&canvas);
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

// pub fn test_draw(canvas: &web_sys::HtmlCanvasElement) {
//     let graphics = driver::Graphics::new(canvas, driver::ColorSpace::Default).unwrap();
//
//     let image = graphics.new_screen_image(800, 600);
//     let shader = graphics
//         .compile_shader(
//             r##"#version 300 es
// precision mediump float;
//
// in vec2 dstPos;
//
// void main() {
//     gl_Position = vec4(dstPos, 0.0, 1.0);
// }"##,
//             r##"#version 300 es
// precision mediump float;
//
// out vec4 fragColor;
//
// void main() {
//     fragColor = vec4(1.0, 1.0, 0.0, 1.0);
// }"##,
//             false,
//             &[],
//         )
//         .unwrap();
//
//     let vertex = [0f32, 0., 0.5, 0., 0., 0.5];
//     let vertex_buffer = unsafe {
//         std::slice::from_raw_parts(
//             vertex.as_ptr() as *const u8,
//             vertex.len() * mem::size_of::<f32>(),
//         )
//     };
//     let indexes = [0u32, 1, 2];
//     let indexes_buffer = unsafe {
//         std::slice::from_raw_parts(
//             indexes.as_ptr() as *const u8,
//             indexes.len() * mem::size_of::<u32>(),
//         )
//     };
//
//     graphics.set_vertices(
//         &driver::GLVertexLayout::new_with_parts(vec![driver::GLVertexLayoutPart::new(2, 8)]),
//         vertex_buffer,
//         indexes_buffer,
//     );

// graphics
//     .draw_triangles(
//         &image,
//         &[],
//         &shader,
//         &[driver::DstRegion::new(
//             crate::utils::Rectangle::new(0, 0, 800, 600),
//             3,
//         )],
//         0,
//         driver::BLEND_DEFAULT,
//         driver::FillRule::FillAll,
//         &[],
//     )
//     .unwrap();
// }
