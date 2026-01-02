use std::sync;

use crate::{DrawOption, GeoM, Image, ImageType, InputState, ui};

#[allow(async_fn_in_trait)]
pub trait Game {
    fn update(&mut self);

    fn draw(&mut self, screen: &Image);

    fn layout(&self) -> (i32, i32);

    fn update_input_state(&mut self, state: InputState);

    fn set_actual_fps(&mut self, _actual_fps: f64) {}
}

pub(super) struct ImplUIGame<G>
where
    G: Game,
{
    game: G,
    offscreen: Option<Image>,
    screen: Option<Image>,
    actual_fps: f64,
}

unsafe impl<G> Sync for ImplUIGame<G> where G: Game {}
unsafe impl<G> Send for ImplUIGame<G> where G: Game {}

impl<G> ImplUIGame<G>
where
    G: Game,
{
    pub(super) fn new(game: G) -> Self {
        Self {
            game,

            offscreen: None,
            screen: None,
            actual_fps: 0.,
        }
    }
}

impl<G> ui::Game for ImplUIGame<G>
where
    G: Game,
{
    fn update(&mut self) {
        self.game.update();
    }

    fn draw_offscreen(&mut self) {
        let Some(offscreen) = self.offscreen.as_ref() else {
            return;
        };
        self.game.draw(offscreen);
    }

    fn draw_final_screen(&mut self, scale: f32, offset_x: f32, offset_y: f32) {
        let mut draw_opt = DrawOption::default();
        draw_opt.set_geom(
            GeoM::default()
                .scale(scale, scale)
                .translate(offset_x, offset_y),
        );

        let Some(offscreen) = self.offscreen.as_ref() else {
            return;
        };
        if let Some(screen) = self.screen.as_mut() {
            screen.draw(offscreen, draw_opt);
        }
    }

    fn set_actual_fps(&mut self, actual_fps: f64) {
        self.actual_fps = actual_fps;
        self.game.set_actual_fps(actual_fps);
    }

    fn update_input_state(&mut self, state: ui::InputState) {
        self.game.update_input_state(state);
    }

    fn layout(&self) -> (i32, i32, f32) {
        let (w, h) = self.game.layout();

        (w, h, 1.)
    }

    fn new_offscreen(
        &mut self,
        width: i32,
        height: i32,
    ) -> sync::Arc<sync::Mutex<Option<ui::Image>>> {
        let image = Image::new(width, height, ImageType::Volatile);
        let result = image.get_ui_image();
        self.offscreen.replace(image);

        result
    }

    fn new_screen(&mut self, width: i32, height: i32) -> sync::Arc<sync::Mutex<Option<ui::Image>>> {
        let image = Image::new(width, height, ImageType::Screen);
        let result = image.get_ui_image();
        self.screen.replace(image);

        result
    }
}
