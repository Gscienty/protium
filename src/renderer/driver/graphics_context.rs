use std::{cell, mem, ptr};

use crate::utils;

use super::{
    BufferID, FramebufferID, ProgramID, RenderbufferID, TextureID, blend, glhandler,
    location_cache, shader_type,
};

struct GraphicsContextData {
    screen_framebuffer: FramebufferID,
    last_framebuffer: FramebufferID,
    last_texture: TextureID,
    last_renderbuffer: RenderbufferID,
    last_viewport_width: i32,
    last_viewport_height: i32,
    last_blend: Option<blend::Blend>,
}

pub(super) struct GraphicsContext {
    location_cache: location_cache::LocationCache,

    data: cell::RefCell<GraphicsContextData>,
}

pub(super) const INVALID_FRAMEBUFFER: FramebufferID = 0xFFFFFFFF;

impl GraphicsContext {
    pub(super) fn new() -> Self {
        Self {
            location_cache: location_cache::LocationCache::new(),

            data: cell::RefCell::new(GraphicsContextData {
                screen_framebuffer: 0,
                last_framebuffer: 0,
                last_texture: 0,
                last_renderbuffer: 0,
                last_viewport_width: 0,
                last_viewport_height: 0,
                last_blend: None,
            }),
        }
    }

    pub(super) fn get_screen_framebuffer(&self) -> FramebufferID {
        let Ok(data) = self.data.try_borrow() else {
            return 0;
        };

        data.screen_framebuffer
    }

    pub(super) fn bind_texture(&self, hdr: &glhandler::GLHandler, texture: TextureID) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };

        if data.last_texture == texture {
            return;
        }
        hdr.bind_texture(web_sys::WebGl2RenderingContext::TEXTURE_2D, texture);
        data.last_texture = texture;
    }

    pub(super) fn bind_renderbuffer(
        &self,
        hdr: &glhandler::GLHandler,
        renderbuffer: RenderbufferID,
    ) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };

        if data.last_renderbuffer == renderbuffer {
            return;
        }
        hdr.bind_renderbuffer(web_sys::WebGl2RenderingContext::RENDERBUFFER, renderbuffer);
        data.last_renderbuffer = renderbuffer;
    }

    pub(super) fn bind_framebuffer(&self, hdr: &glhandler::GLHandler, framebuffer: FramebufferID) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };

        if data.last_framebuffer == framebuffer {
            return;
        }
        hdr.bind_framebuffer(web_sys::WebGl2RenderingContext::FRAMEBUFFER, framebuffer);
        data.last_framebuffer = framebuffer;
    }

    pub(super) fn set_viewport(
        &self,
        hdr: &glhandler::GLHandler,
        framebuffer: FramebufferID,
        width: i32,
        height: i32,
    ) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };

        if data.last_framebuffer != framebuffer {
            hdr.bind_framebuffer(web_sys::WebGl2RenderingContext::FRAMEBUFFER, framebuffer);
            data.last_framebuffer = framebuffer;
        }
        if data.last_viewport_width == width && data.last_viewport_height == height {
            return;
        }
        hdr.viewpoint(0, 0, width, height);

        if framebuffer == data.screen_framebuffer {
            data.last_viewport_width = 0;
            data.last_viewport_height = 0;
        } else {
            data.last_viewport_width = width;
            data.last_viewport_height = height;
        }
    }

    pub(super) fn new_screen_framebuffer(&self) -> FramebufferID {
        let Ok(data) = self.data.try_borrow() else {
            return INVALID_FRAMEBUFFER;
        };
        data.screen_framebuffer
    }

    pub(super) fn reset(&self, hdr: &glhandler::GLHandler) {
        self.location_cache.clear();

        hdr.enable(web_sys::WebGl2RenderingContext::BLEND);
        hdr.enable(web_sys::WebGl2RenderingContext::SCISSOR_TEST);

        self.blend(hdr, blend::BLEND_DEFAULT);

        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };

        data.last_texture = 0;
        data.last_framebuffer = INVALID_FRAMEBUFFER;
        data.last_viewport_width = 0;
        data.last_viewport_height = 0;
        data.last_blend = None;
        data.screen_framebuffer =
            hdr.get_integer(web_sys::WebGl2RenderingContext::FRAMEBUFFER_BINDING) as FramebufferID;
    }

    pub(super) fn reset_last_texture(&self) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };
        data.last_texture = 0;
    }

    pub(super) fn blend(&self, hdr: &glhandler::GLHandler, blend: blend::Blend) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };

        if data.last_blend.eq(&Some(blend)) {
            return;
        }
        data.last_blend = Some(blend);

        hdr.blend_func_separate(
            blend.factor_source_rgb.into(),
            blend.factor_destination_rgb.into(),
            blend.factor_source_alpha.into(),
            blend.factor_destination_alpha.into(),
        );
        hdr.blend_equation_separate(blend.op_rgb.into(), blend.op_alpha.into());
    }

    pub(super) fn new_texture(
        &self,
        hdr: &glhandler::GLHandler,
        width: i32,
        height: i32,
    ) -> Result<TextureID, ()> {
        let Some(texture) = hdr.create_texture() else {
            return Err(());
        };
        if texture == 0 {
            return Err(());
        }
        self.bind_texture(hdr, texture);

        hdr.tex_parameter(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            web_sys::WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            web_sys::WebGl2RenderingContext::NEAREST as i32,
        );
        hdr.tex_parameter(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            web_sys::WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            web_sys::WebGl2RenderingContext::NEAREST as i32,
        );
        hdr.tex_parameter(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            web_sys::WebGl2RenderingContext::TEXTURE_WRAP_S,
            web_sys::WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        hdr.tex_parameter(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            web_sys::WebGl2RenderingContext::TEXTURE_WRAP_T,
            web_sys::WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        hdr.pixel_store(web_sys::WebGl2RenderingContext::UNPACK_ALIGNMENT, 4);

        hdr.tex_image_2d(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            0,
            web_sys::WebGl2RenderingContext::RGBA as i32,
            width,
            height,
            web_sys::WebGl2RenderingContext::RGBA,
            web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
        );

        Ok(texture)
    }

    pub(super) fn framebuffer_pixels(
        &self,
        hdr: &glhandler::GLHandler,
        framebuffer: FramebufferID,
        region: &utils::Rectangle,
    ) -> Vec<u8> {
        hdr.flush();
        self.bind_framebuffer(hdr, framebuffer);

        let x = region.min.x;
        let y = region.min.y;
        let width = region.get_width();
        let height = region.get_height();

        let mut data = vec![0u8; (4 * width * height) as usize];
        hdr.read_pixels(
            &mut data,
            x,
            y,
            width,
            height,
            web_sys::WebGl2RenderingContext::RGBA,
            web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
        );

        data
    }

    pub(super) fn delete_texture(&self, hdr: &glhandler::GLHandler, texture: TextureID) {
        hdr.delete_texture(texture);

        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };
        if data.last_texture == texture {
            data.last_texture = 0;
        }
    }

    pub(super) fn new_renderbuffer(
        &self,
        hdr: &glhandler::GLHandler,
        width: i32,
        height: i32,
    ) -> Result<RenderbufferID, ()> {
        let Some(renderbuffer) = hdr.create_renderbuffer() else {
            return Err(());
        };
        if renderbuffer == 0 {
            return Err(());
        }

        self.bind_renderbuffer(hdr, renderbuffer);
        hdr.renderbuffer_storage(
            web_sys::WebGl2RenderingContext::RENDERBUFFER,
            web_sys::WebGl2RenderingContext::STENCIL_INDEX8,
            width,
            height,
        );

        Ok(renderbuffer)
    }

    pub(super) fn delete_renderbuffer(
        &self,
        hdr: &glhandler::GLHandler,
        renderbuffer: RenderbufferID,
    ) {
        hdr.delete_renderbuffer(renderbuffer);

        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };
        if data.last_renderbuffer == renderbuffer {
            data.last_renderbuffer = 0;
        }
    }

    pub(super) fn new_framebuffer(
        &self,
        hdr: &glhandler::GLHandler,
        texture: TextureID,
    ) -> Result<FramebufferID, ()> {
        let Some(framebuffer) = hdr.creae_framebuffer() else {
            return Err(());
        };
        if framebuffer == 0 {
            return Err(());
        }
        self.bind_framebuffer(hdr, framebuffer);

        hdr.framebuffer_texture_2d(
            web_sys::WebGl2RenderingContext::FRAMEBUFFER,
            web_sys::WebGl2RenderingContext::COLOR_ATTACHMENT0,
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            texture,
            0,
        );

        Ok(framebuffer)
    }

    pub(super) fn bind_stencil_buffer(
        &self,
        hdr: &glhandler::GLHandler,
        framebuffer: FramebufferID,
        renderbuffer: RenderbufferID,
    ) {
        self.bind_framebuffer(hdr, framebuffer);
        hdr.framebuffer_renderbuffer(
            web_sys::WebGl2RenderingContext::FRAMEBUFFER,
            web_sys::WebGl2RenderingContext::STENCIL_ATTACHMENT,
            web_sys::WebGl2RenderingContext::RENDERBUFFER,
            renderbuffer,
        );
    }

    pub(super) fn delete_framebuffer(
        &self,
        hdr: &glhandler::GLHandler,
        framebuffer: FramebufferID,
    ) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };
        if data.screen_framebuffer == framebuffer {
            return;
        }
        if data.last_framebuffer == framebuffer {
            data.last_framebuffer = INVALID_FRAMEBUFFER;
            data.last_viewport_width = 0;
            data.last_viewport_height = 0;
        }

        hdr.delete_framebuffer(framebuffer);
    }

    pub(super) fn delete_program(self, hdr: &glhandler::GLHandler, program: ProgramID) {
        self.location_cache.delete_program(program);
        if !hdr.is_program(program) {
            return;
        }
        hdr.delete_program(program);
    }

    pub(super) fn uniform_int(
        &self,
        hdr: &glhandler::GLHandler,
        program: ProgramID,
        name: &str,
        data: i32,
    ) -> bool {
        let location = if let Some(location) = self.location_cache.get_location(program, name) {
            location
        } else {
            let Some(location) = hdr.get_uniform_location(program, name) else {
                return false;
            };
            self.location_cache.set_location(program, name, location);

            location
        };

        hdr.uniform_1i(program, location, data);

        true
    }

    pub(super) fn uniform(
        &self,
        hdr: &glhandler::GLHandler,
        program: ProgramID,
        name: &str,
        typ: &shader_type::ShaderType,
        data: &[u8],
    ) -> bool {
        let location = if let Some(location) = self.location_cache.get_location(program, name) {
            location
        } else {
            let Some(location) = hdr.get_uniform_location(program, name) else {
                return false;
            };
            self.location_cache.set_location(program, name, location);

            location
        };

        let base = if matches!(typ.main, shader_type::ShaderBasicType::Array) {
            typ.sub[0].main
        } else {
            typ.main
        };

        match base {
            shader_type::ShaderBasicType::Float => {
                hdr.uniform_1fv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::Int => {
                hdr.uniform_1iv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::Vec2 => {
                hdr.uniform_2fv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::IVec2 => {
                hdr.uniform_2iv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::Mat2 => {
                hdr.uniform_matrix_2fv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::Vec3 => {
                hdr.uniform_3fv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::IVec3 => {
                hdr.uniform_3iv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::Mat3 => {
                hdr.uniform_matrix_3fv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::Vec4 => {
                hdr.uniform_4fv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::IVec4 => {
                hdr.uniform_4iv(program, location, Self::uniform_value_of(data));
            }
            shader_type::ShaderBasicType::Mat4 => {
                hdr.uniform_matrix_4fv(program, location, Self::uniform_value_of(data));
            }
            _ => {}
        }

        true
    }

    fn uniform_value_of<T>(data: &[u8]) -> &[T] {
        unsafe {
            &*ptr::slice_from_raw_parts(data.as_ptr() as *const T, data.len() / mem::size_of::<T>())
        }
    }

    pub(super) fn new_array_buffer(&self, hdr: &glhandler::GLHandler, size: i32) -> BufferID {
        let Some(buffer) = hdr.create_buffer() else {
            return 0;
        };
        hdr.bind_buffer(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, buffer);
        hdr.buffer_init(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            size,
            web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        buffer
    }

    pub(super) fn new_element_array_buffer(
        &self,
        hdr: &glhandler::GLHandler,
        size: i32,
    ) -> BufferID {
        let Some(buffer) = hdr.create_buffer() else {
            return 0;
        };
        hdr.bind_buffer(
            web_sys::WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            buffer,
        );
        hdr.buffer_init(
            web_sys::WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            size,
            web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        buffer
    }
}
