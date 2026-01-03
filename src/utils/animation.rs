use crate::image;

use super::Rectangle;

pub struct Animation {
    atlas: image::Image,
    frames: Vec<image::Image>,
    fps: f32,

    paused: bool,
    tick: f32,
    current_index: usize,
}

impl Animation {
    pub fn new(atlas: image::Image, fps: f32, blocks: &[Rectangle], pingpong: bool) -> Self {
        let mut frames = Vec::with_capacity(blocks.len() * if pingpong { 2 } else { 1 });
        for &block in blocks.iter() {
            let Some(frame) = atlas.sub(block) else {
                continue;
            };

            frames.push(frame);
        }

        if pingpong {
            for &block in blocks.iter().skip(1).rev() {
                let Some(frame) = atlas.sub(block) else {
                    continue;
                };

                frames.push(frame);
            }
        }

        Self {
            atlas,
            frames,
            fps,

            paused: false,
            tick: 0.,
            current_index: 0,
        }
    }

    pub fn pause_at(&mut self, index: usize) {
        self.current_index = index;
        self.paused = true;
    }

    pub fn play_at(&mut self, index: usize) {
        self.current_index = index;
        self.paused = false;
    }

    pub fn update(&mut self) {
        if self.paused {
            return;
        }

        self.tick += self.fps / 60.;
        self.current_index = self.tick.floor() as usize;
        if self.current_index >= self.frames.len() {
            self.tick = 0.;
            self.current_index = 0;
        }
    }

    pub fn get_current_frame(&self) -> Option<&image::Image> {
        self.frames.get(self.current_index)
    }

    pub fn deallocate(&self) {
        self.atlas.deallocate();
    }
}
