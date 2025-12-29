#[allow(unused)]
mod atlas;
mod renderer;
mod utils;

extern crate console_error_panic_hook;
use std::panic;

use atlas::{AtlasDrawingQuatEntity, AtlasDrawingSource};

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

    let mut source_img = atlas::Image::new(2, 2, atlas::ImageType::Regular);
    let mut screen = atlas::Image::new(800, 600, atlas::ImageType::Screen);

    atlas::begin_frame();

    source_img.write_pixels(
        vec![
            0xff, 0x00, 0x00, 0xff, //
            0x00, 0xff, 0x00, 0xff, //
            0x00, 0x00, 0xff, 0xff, //
            0x00, 0xff, 0xff, 0xff, //
        ],
        utils::Rectangle::new(0, 0, 2, 2),
    );

    if let Some(drawing) = screen.draw(
        utils::Rectangle::new(0, 0, 600, 600),
        AtlasDrawingSource::new(&source_img, utils::Rectangle::new(0, 0, 1, 1)),
    ) {
        drawing.process_variable(|mut variable| {
            variable
                .quad([
                    AtlasDrawingQuatEntity::new(
                        utils::Point::new(0, 0),
                        utils::Point::new(1, 1),
                        (1.0, 1.0, 1.0, 1.0),
                    ),
                    AtlasDrawingQuatEntity::new(
                        utils::Point::new(0, 600),
                        utils::Point::new(1, 2),
                        (1.0, 1.0, 1.0, 1.0),
                    ),
                    AtlasDrawingQuatEntity::new(
                        utils::Point::new(600, 0),
                        utils::Point::new(2, 1),
                        (1.0, 1.0, 1.0, 1.0),
                    ),
                    AtlasDrawingQuatEntity::new(
                        utils::Point::new(600, 600),
                        utils::Point::new(2, 2),
                        (1.0, 1.0, 1.0, 1.0),
                    ),
                ])
                .extend_index(vec![0, 1, 2, 1, 2, 3]);
        });
        drawing.draw();
    }
    atlas::end_frame();
    atlas::flush(true);
}
