#[allow(unused)]
mod atlas;
mod renderer;
mod utils;

extern crate console_error_panic_hook;
use std::panic;

#[wasm_bindgen_futures::wasm_bindgen::prelude::wasm_bindgen]
pub async fn run(canvas: web_sys::HtmlCanvasElement) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    test_draw(&canvas);
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub fn test_draw(canvas: &web_sys::HtmlCanvasElement) {
    let renderer = renderer::Renderer::create(canvas, renderer::ColorSpace::Default, 80).unwrap();
    let shader = renderer.create_shader(
        r##"#version 300 es
 precision mediump float;

 in vec2 dstPos;

 void main() {
     gl_Position = vec4(dstPos, 0.0, 1.0);
 }"##
        .to_string(),
        r##"#version 300 es
 precision mediump float;

 out vec4 fragColor;

 void main() {
     fragColor = vec4(1.0, 1.0, 0.0, 1.0);
 }"##
        .to_string(),
        false,
        renderer::GLVertexLayout::new_with_parts(vec![renderer::GLVertexLayoutPart::new(2, 8)]),
        vec![],
    );

    let image = renderer.create_image(800, 600, true);

    let vertex = [0f32, 0., 0.5, 0., 0., 0.5];
    let vertex_buffer = unsafe {
        std::slice::from_raw_parts(
            vertex.as_ptr() as *const u8,
            vertex.len() * std::mem::size_of::<f32>(),
        )
    };

    image
        .drawing(utils::Rectangle::new(0, 0, 800, 600), &shader)
        .append_vertices(vertex_buffer.to_owned())
        .append_indexes(vec![0, 1, 2])
        .draw();

    renderer.flush(true);
}
