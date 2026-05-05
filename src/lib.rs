mod atlas;
mod color_scale;
mod colorm;
mod draw_option;
mod game;
mod geom;
mod image;
mod renderer;
mod run;
mod ui;

pub mod text;
pub mod utils;

pub use atlas::ImageType;
pub use color_scale::ColorScale;
pub use colorm::ColorM;
pub use draw_option::DrawOption;
pub use game::Game;
pub use geom::GeoM;
pub use image::Image;
pub use renderer::{
    BLEND_CLEAR, BLEND_COPY, BLEND_DEFAULT, BLEND_SOURCE_OVER, ColorSpace, FillRule,
};
pub use run::{init, run_game};
pub use ui::{InputState, Key, MouseButton};

extern crate console_error_panic_hook;

// #[wasm_bindgen_futures::wasm_bindgen::prelude::wasm_bindgen]
// pub async fn run(canvas: web_sys::HtmlCanvasElement) {
//     panic::set_hook(Box::new(console_error_panic_hook::hook));
//
//     test_draw(&canvas).await;
// }

#[wasm_bindgen::prelude::wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

// struct MockGame {
//     font_img: Image,
// }
//
// impl Game for MockGame {
//     fn draw(&mut self, screen: &Image) {
//         let mut option = DrawOption::default();
//         option.set_geom(GeoM::default().translate(400., 300.));
//         screen.draw(&self.font_img, option);
//     }
//
//     fn update(&mut self) {}
//
//     fn layout(&self) -> (i32, i32) {
//         (800, 600)
//     }
//
//     fn update_input_state(&mut self, state: InputState) {}
// }
//
// async fn test_draw(canvas: &web_sys::HtmlCanvasElement) {
//     init(canvas, ColorSpace::Default, 80);
//
//     let resp = reqwest_wasm::Client::default()
//         .request(
//             reqwest_wasm::Method::GET,
//             "http://localhost:8000/OpenSans-Light.ttf",
//         )
//         .send()
//         .await;
//
//     let ttf = resp.unwrap().bytes().await.unwrap();
//     let mut font_set = text::FontSet::new();
//     font_set.add_font("sans", &ttf);
//     let mut span = font_set.new_span("sans").unwrap();
//     span.append("A", 48., (0., 0., 0., 1.));
//
//     let font_img = span.draw().unwrap();
//     // atlas::begin_frame();
//     //
//     // let screen = Image::new(800, 600, ImageType::Screen);
//     //
//     // let mut option = DrawOption::default();
//     // option.set_geom(GeoM::default().translate(300., 300.));
//     // screen.draw(&font_img, option);
//     //
//     // atlas::end_frame();
//     //
//     // atlas::flush(true);
//
//     run_game(MockGame { font_img })
// }
