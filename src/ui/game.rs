use super::{image, input_state};
use std::sync;

pub trait Game: Send + Sync {
    fn set_actual_fps(&mut self, actual_fps: f64);

    fn update(&mut self);

    fn new_offscreen(
        &mut self,
        width: i32,
        height: i32,
    ) -> sync::Arc<sync::Mutex<Option<image::Image>>>;

    fn new_screen(
        &mut self,
        width: i32,
        height: i32,
    ) -> sync::Arc<sync::Mutex<Option<image::Image>>>;

    fn draw_offscreen(&mut self);

    fn draw_final_screen(&mut self, scale: f32, offset_x: f32, offset_y: f32);

    fn layout(&self) -> (i32, i32, f32);

    fn update_input_state(&mut self, state: input_state::InputState);
}
