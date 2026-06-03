use std::{cell, rc, sync, time};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::atlas;

use super::{clock, fps, game, image, input_state};

struct UIContext<G>
where
    G: game::Game,
{
    game: G,

    offscreen: sync::Arc<sync::Mutex<Option<image::Image>>>,
    screen: sync::Arc<sync::Mutex<Option<image::Image>>>,

    screen_width: i32,
    screen_height: i32,
    offscreen_width: i32,
    offscreen_height: i32,

    offscreen_modified: sync::Arc<sync::Mutex<bool>>,
    skip_count: usize,
}

const MAX_SKIP_COUNT: usize = 4;

pub(crate) struct EventLoop<G>
where
    G: game::Game,
{
    context: tokio::sync::Mutex<UIContext<G>>,

    fps_mode: fps::FPSMode,
    last_flush_buffer_time: sync::atomic::AtomicU64,
    clock: clock::Clock,
}

type LoopClosureType = rc::Rc<cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut()>>>>;

impl<G> EventLoop<G>
where
    G: game::Game + 'static,
{
    pub fn new(game: G) -> sync::Arc<Self> {
        sync::Arc::new(Self {
            context: tokio::sync::Mutex::new(UIContext {
                game,
                offscreen: sync::Arc::new(sync::Mutex::new(None)),
                screen: sync::Arc::new(sync::Mutex::new(None)),

                screen_width: 0,
                screen_height: 0,
                offscreen_width: 0,
                offscreen_height: 0,

                offscreen_modified: sync::Arc::new(sync::Mutex::new(false)),
                skip_count: 0,
            }),

            fps_mode: fps::FPSMode::On,
            last_flush_buffer_time: sync::atomic::AtomicU64::new(0),
            clock: clock::Clock::new(),
        })
    }

    pub fn run(self: &sync::Arc<Self>) -> Result<(), ()> {
        let loop_func = rc::Rc::new(cell::RefCell::new(None));
        *loop_func.borrow_mut() = self.clone().build_loop_closure(loop_func.clone());

        Self::request_animation_frame(loop_func.borrow().as_ref().unwrap())?;

        Ok(())
    }

    fn request_animation_frame(f: &wasm_bindgen::prelude::Closure<dyn FnMut()>) -> Result<(), ()> {
        let window = web_sys::window().ok_or(())?;
        window
            .request_animation_frame(f.as_ref().unchecked_ref())
            .map_err(|_| ())?;

        Ok(())
    }

    fn build_loop_closure(
        self: sync::Arc<Self>,
        loop_closure: LoopClosureType,
    ) -> Option<wasm_bindgen::prelude::Closure<dyn FnMut()>> {
        Some(wasm_bindgen::prelude::Closure::new(move || {
            let self_clone = self.clone();
            spawn_local(async move {
                self_clone.update().await;
            });
            let _ = Self::request_animation_frame(loop_closure.borrow().as_ref().unwrap());
        }))
    }

    async fn update(&self) {
        let (frames, actual_fps) = self.clock.update_frame();

        let need_flush_buffer = if let Ok(mut ctx) = self.context.try_lock() {
            ctx.game.set_actual_fps(actual_fps);

            atlas::begin_frame();
            let need_flush_buffer = self.update_inner(&mut ctx, frames).await;
            atlas::end_frame();

            need_flush_buffer
        } else {
            return;
        };

        self.flush_buffer_or_wait(need_flush_buffer).await;
    }

    async fn update_inner(&self, ctx: &mut UIContext<G>, frames: u64) -> bool {
        let (w, h) = self.reset_layout(ctx);
        if w == 0 || h == 0 {
            return false;
        }

        if let Some(input_state) = input_state::process_input_state(|state| state.update_state()) {
            ctx.game.update_input_state(input_state);
        }

        for _ in 0..frames {
            ctx.game.update().await;
        }
        if frames > 0 {
            self.draw_game(ctx)
        } else {
            false
        }
    }

    fn draw_game(&self, ctx: &mut UIContext<G>) -> bool {
        if let Ok(mut modified) = ctx.offscreen_modified.lock() {
            *modified = false;
        }

        let renew_offscreen = if let Ok(mut offscreen) = ctx.offscreen.lock() {
            if let Some(offscreen) = offscreen.as_mut() {
                offscreen.clear();
                false
            } else {
                true
            }
        } else {
            true
        };

        if renew_offscreen {
            ctx.offscreen = self.new_offscreen(ctx);
        }
        ctx.game.draw_offscreen();

        let modified = if let Ok(modified) = ctx.offscreen_modified.lock() {
            *modified
        } else {
            false
        };
        if !modified {
            if ctx.skip_count < MAX_SKIP_COUNT {
                ctx.skip_count += 1;
            }
        } else {
            ctx.skip_count = 0;
        }

        if ctx.skip_count >= MAX_SKIP_COUNT {
            return false;
        }

        if let Ok(mut screen) = ctx.screen.lock()
            && let Some(screen) = screen.as_mut()
        {
            screen.clear();
        }

        let (scale, offset_x, offset_y) = ctx.screen_scale_and_offsets();
        ctx.game.draw_final_screen(scale, offset_x, offset_y);
        true
    }

    fn reset_layout(&self, ctx: &mut UIContext<G>) -> (i32, i32) {
        let (width, height, scale_factor) = ctx.game.layout();

        let screen_width = (width as f32 * scale_factor) as i32;
        let screen_height = (height as f32 * scale_factor) as i32;
        if ctx.screen_width != screen_width || ctx.screen_height != screen_height {
            ctx.skip_count = 0;
        }

        ctx.screen_width = screen_width;
        ctx.screen_height = screen_height;
        ctx.offscreen_width = width;
        ctx.offscreen_height = height;

        let renew_screen = {
            let Ok(mut screen_guard) = ctx.screen.lock() else {
                return (ctx.offscreen_width, ctx.offscreen_height);
            };

            if let Some(screen) = screen_guard.as_mut() {
                let (width, height) = screen.get_size();
                if width != ctx.screen_width || height != ctx.screen_height {
                    screen.deallocate();
                    *screen_guard = None;
                }
            }
            screen_guard.is_none()
        };
        if renew_screen {
            ctx.screen = ctx.game.new_screen(ctx.screen_width, ctx.screen_height);
        }

        let renew_offscreen = {
            let Ok(mut offscreen_guard) = ctx.offscreen.lock() else {
                return (ctx.offscreen_width, ctx.offscreen_height);
            };

            if let Some(offscreen) = offscreen_guard.as_mut() {
                let (width, height) = offscreen.get_size();
                if width != ctx.offscreen_width || height != ctx.offscreen_height {
                    offscreen.deallocate();
                    *offscreen_guard = None;
                }
            }
            offscreen_guard.is_none()
        };
        if renew_offscreen {
            ctx.offscreen = self.new_offscreen(ctx);
        }

        (ctx.offscreen_width, ctx.offscreen_height)
    }

    fn new_offscreen(
        &self,
        ctx: &mut UIContext<G>,
    ) -> sync::Arc<sync::Mutex<Option<image::Image>>> {
        let offscreen = ctx
            .game
            .new_offscreen(ctx.offscreen_width, ctx.offscreen_height);

        let modified = ctx.offscreen_modified.clone();

        if let Ok(mut offscreen) = offscreen.lock()
            && let Some(offscreen) = offscreen.as_mut()
        {
            offscreen.set_modify_callback(Some(Box::new(move || {
                if let Ok(mut modified) = modified.lock() {
                    *modified = true;
                }
            })));
        };

        offscreen
    }

    pub(crate) async fn flush_buffer_or_wait(&self, need_flush_buffer: bool) {
        let now = chrono::Local::now().timestamp_millis() as u64;

        if need_flush_buffer {
            atlas::flush(need_flush_buffer);
        }

        let wait_time = if !need_flush_buffer {
            1000 / 60
        } else if matches!(self.fps_mode, fps::FPSMode::On) {
            1
        } else {
            0
        };

        if wait_time > 0 {
            let duration = now
                - self
                    .last_flush_buffer_time
                    .load(sync::atomic::Ordering::Acquire);
            if wait_time > duration {
                tokio_with_wasm::alias::time::sleep(time::Duration::from_millis(
                    wait_time - duration,
                ))
                .await;
            }
        }

        self.last_flush_buffer_time
            .store(now, sync::atomic::Ordering::Release);
    }
}

impl<G> UIContext<G>
where
    G: game::Game,
{
    fn screen_scale_and_offsets(&self) -> (f32, f32, f32) {
        let scale_x = self.screen_width as f32 / self.offscreen_width as f32;
        let scale_y = self.screen_height as f32 / self.offscreen_height as f32;
        let scale = scale_x.min(scale_y);
        let width = self.offscreen_width as f32 * scale;
        let height = self.offscreen_height as f32 * scale;
        let offset_x = (self.screen_width as f32 - width) / 2.;
        let offset_y = (self.screen_height as f32 - height) / 2.;

        (scale, offset_x, offset_y)
    }
}
