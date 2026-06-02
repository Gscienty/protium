use std::{cell::Cell, sync};

use crate::utils;

use super::{FramebufferID, RenderbufferID, TextureID, graphics, pixels_args};

pub(crate) struct Image {
    texture: TextureID,
    framebuffer: Cell<Option<FramebufferID>>,
    stencil: Cell<Option<RenderbufferID>>,

    width: i32,
    height: i32,

    is_screen: bool,
}

impl Image {
    pub(super) fn new(width: i32, height: i32, texture: TextureID, is_screen: bool) -> Self {
        Self {
            texture,
            framebuffer: Cell::new(None),
            stencil: Cell::new(None),

            width,
            height,

            is_screen,
        }
    }

    #[inline(always)]
    pub(crate) const fn get_texture(&self) -> TextureID {
        self.texture
    }

    pub(crate) fn dispose(&self, graphics: &graphics::Graphics) {
        if let Some(framebuffer) = self.framebuffer.get() {
            graphics.get_handler().delete_framebuffer(framebuffer);
        }
        if self.texture != 0 {
            graphics.get_handler().delete_texture(self.texture);
        }
        if let Some(stencil) = self.stencil.get() {
            graphics.get_handler().delete_renderbuffer(stencil);
        }
    }

    pub(crate) const fn viewport_size(&self) -> (i32, i32) {
        if self.is_screen {
            (self.width, self.height)
        } else {
            (
                Self::internal_image_size(self.width),
                Self::internal_image_size(self.height),
            )
        }
    }

    pub(crate) fn read_pixels<'a>(
        &self,
        graphics: &graphics::Graphics,
        args: impl Iterator<Item = &'a utils::Rectangle>,
    ) -> Result<Vec<sync::Arc<[u8]>>, ()> {
        let framebuffer = self.ensure_framebuffer(graphics)?;

        let result = args
            .map(|region| {
                graphics
                    .get_context()
                    .framebuffer_pixels(graphics.get_handler(), framebuffer, region)
                    .into()
            })
            .collect();

        Ok(result)
    }

    pub(crate) fn write_pixels(
        &self,
        graphics: &graphics::Graphics,
        args: impl Iterator<Item = pixels_args::PixelsArgs>,
    ) -> Result<(), ()> {
        if self.is_screen {
            return Err(());
        }

        let mut first = true;
        for arg in args {
            if first {
                if graphics.swap_draw_called(false) {
                    graphics.get_handler().flush();
                }
                graphics
                    .get_context()
                    .bind_texture(graphics.get_handler(), self.texture);
                first = false;
            }

            let (xoffset, yoffset, width, height) =
                (arg.get_x(), arg.get_y(), arg.get_width(), arg.get_height());

            graphics.get_handler().tex_sub_image_2d(
                web_sys::WebGl2RenderingContext::TEXTURE_2D,
                0,
                xoffset,
                yoffset,
                width,
                height,
                web_sys::WebGl2RenderingContext::RGBA,
                web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
                arg.get_pixels(),
            );
        }

        Ok(())
    }

    pub(super) fn set_viewport(&self, graphics: &graphics::Graphics) -> Result<(), ()> {
        let framebuffer = self.ensure_framebuffer(graphics)?;
        let (width, height) = self.viewport_size();
        graphics
            .get_context()
            .set_viewport(graphics.get_handler(), framebuffer, width, height);

        Ok(())
    }

    fn ensure_framebuffer(&self, graphics: &graphics::Graphics) -> Result<FramebufferID, ()> {
        if let Some(framebuffer) = self.framebuffer.get() {
            return Ok(framebuffer);
        }

        if self.is_screen {
            let framebuffer = graphics.get_context().new_screen_framebuffer();
            self.framebuffer.replace(Some(framebuffer));

            return Ok(framebuffer);
        }

        let framebuffer = graphics
            .get_context()
            .new_framebuffer(graphics.get_handler(), self.texture)?;
        self.framebuffer.replace(Some(framebuffer));

        Ok(framebuffer)
    }

    pub(super) fn ensure_stencil_buffer(&self, graphics: &graphics::Graphics) -> Result<(), ()> {
        if self.stencil.get().is_some() {
            return Ok(());
        }

        let framebuffer = self.ensure_framebuffer(graphics)?;

        let (width, height) = self.viewport_size();
        let renderbuffer =
            graphics
                .get_context()
                .new_renderbuffer(graphics.get_handler(), width, height)?;
        self.stencil.replace(Some(renderbuffer));

        graphics.get_context().bind_stencil_buffer(
            graphics.get_handler(),
            framebuffer,
            renderbuffer,
        );

        Ok(())
    }

    pub(crate) const fn internal_image_size(x: i32) -> i32 {
        if x < 16 {
            16
        } else {
            let mut r = 1;
            while r < x {
                r <<= 1;
            }

            r
        }
    }
}
