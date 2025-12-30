mod atlas;
mod renderer;
mod ui;
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
    atlas::init_material(canvas, renderer::ColorSpace::Default, 80);

    let mut screen = ui::Image::new(800, 600, atlas::ImageType::Screen);

    atlas::begin_frame();

    screen.fill(utils::Rectangle::new(100, 300, 300, 300), (1., 0., 1., 1.));

    atlas::end_frame();
    atlas::flush(true);
}
