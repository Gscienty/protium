use std::{cell, collections, mem};

use super::{
    ProgramID, TextureID, blend, color_space, dst_region, fill_rule, glhandler, glprogram,
    graphics_context, image, shader, shader_type, uniform_variable,
};

pub(crate) struct Graphics {
    handler: glhandler::GLHandler,

    program_state: glprogram::ProgramState,
    context: graphics_context::GraphicsContext,

    draw_called: cell::Cell<bool>,
}

unsafe impl Send for Graphics {}

impl Graphics {
    pub(crate) fn new(
        canvas: &web_sys::HtmlCanvasElement,
        space: color_space::ColorSpace,
    ) -> Result<Self, ()> {
        let handler = glhandler::GLHandler::new(canvas, space)?;
        handler.clear_color(0., 0., 0., 1.);
        handler.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        Ok(Self {
            handler,

            program_state: glprogram::ProgramState::new(),
            context: graphics_context::GraphicsContext::new(),

            draw_called: cell::Cell::new(false),
        })
    }

    pub(crate) fn new_image(&self, width: i32, height: i32) -> Result<image::Image, ()> {
        let texture = self.context.new_texture(&self.handler, width, height)?;

        Ok(image::Image::new(width, height, texture, false))
    }

    pub(crate) fn new_screen_image(&self, width: i32, height: i32) -> image::Image {
        image::Image::new(width, height, 0, true)
    }

    pub(crate) fn flush(&self, preset: bool) {
        self.handler.flush();
        if preset {
            self.program_state.reset_last_uniforms();
        }
    }

    pub(crate) fn get_max_texture_size(&self) -> i32 {
        self.handler
            .get_integer(web_sys::WebGl2RenderingContext::MAX_TEXTURE_SIZE)
    }

    #[inline(always)]
    pub(crate) fn initialize(&self) {
        self.reset();
    }

    #[inline(always)]
    pub(crate) fn reset(&self) {
        self.context.reset(&self.handler);
        self.program_state.reset(&self.handler);
    }

    #[inline(always)]
    pub(crate) fn set_vertices(
        &self,
        layout: &glprogram::GLVertexLayout,
        vertices: &[f32],
        indexes: &[u32],
    ) {
        self.program_state
            .set_vertices(&self.handler, layout, vertices, indexes);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_triangles<'a>(
        &self,
        dst: &image::Image,
        textures: impl Iterator<Item = (&'a str, TextureID)>,
        shader: &shader::Shader,
        dst_regions: impl Iterator<Item = &'a dst_region::DstRegion>,
        blend: blend::Blend,
        fill_rule: fill_rule::FillRule,
        uniforms: &[u8],
    ) -> Result<(), ()> {
        self.swap_draw_called(true);

        dst.set_viewport(self)?;
        self.context.blend(&self.handler, blend);

        let mut offset = 0;
        let mut uniform_variables = Vec::new();
        for (name, typ) in shader.iter_uniform_types() {
            let bytes = typ.bytes_count();
            uniform_variables.push(uniform_variable::UniformVariable::new(
                name,
                &uniforms[offset..offset + bytes],
                typ.clone(),
            ));
            offset += bytes;
        }

        self.use_program(shader.get_program(), textures, &uniform_variables);

        if !matches!(fill_rule, fill_rule::FillRule::FillAll) {
            dst.ensure_stencil_buffer(self)?;
            self.handler
                .enable(web_sys::WebGl2RenderingContext::STENCIL_TEST);
        }

        let mut index_offset = 0;
        for dst_region in dst_regions {
            let (x, y, width, height) = (
                dst_region.get_x(),
                dst_region.get_y(),
                dst_region.get_width(),
                dst_region.get_height(),
            );
            self.handler.scissor(x, y, width, height);

            match fill_rule {
                fill_rule::FillRule::NonZero => {
                    self.handler
                        .clear(web_sys::WebGl2RenderingContext::STENCIL_BUFFER_BIT);
                    self.handler
                        .stencil_func(web_sys::WebGl2RenderingContext::ALWAYS, 0x00, 0xff);
                    self.handler.stencil_op_separate(
                        web_sys::WebGl2RenderingContext::FRONT,
                        web_sys::WebGl2RenderingContext::KEEP,
                        web_sys::WebGl2RenderingContext::KEEP,
                        web_sys::WebGl2RenderingContext::INCR_WRAP,
                    );
                    self.handler.stencil_op_separate(
                        web_sys::WebGl2RenderingContext::BACK,
                        web_sys::WebGl2RenderingContext::KEEP,
                        web_sys::WebGl2RenderingContext::KEEP,
                        web_sys::WebGl2RenderingContext::DECR_WRAP,
                    );
                    self.handler.color_mask(false, false, false, false);
                    self.handler.draw_elements(
                        web_sys::WebGl2RenderingContext::TRIANGLES,
                        dst_region.get_count(),
                        web_sys::WebGl2RenderingContext::UNSIGNED_INT,
                        index_offset * mem::size_of::<u32>() as i32,
                    );
                }
                fill_rule::FillRule::EvenOdd => {
                    self.handler
                        .clear(web_sys::WebGl2RenderingContext::STENCIL_BUFFER_BIT);
                    self.handler
                        .stencil_func(web_sys::WebGl2RenderingContext::ALWAYS, 0x00, 0xff);
                    self.handler.stencil_op_separate(
                        web_sys::WebGl2RenderingContext::FRONT_AND_BACK,
                        web_sys::WebGl2RenderingContext::KEEP,
                        web_sys::WebGl2RenderingContext::KEEP,
                        web_sys::WebGl2RenderingContext::INVERT,
                    );
                    self.handler.color_mask(false, false, false, false);
                    self.handler.draw_elements(
                        web_sys::WebGl2RenderingContext::TRIANGLES,
                        dst_region.get_count(),
                        web_sys::WebGl2RenderingContext::UNSIGNED_INT,
                        index_offset * mem::size_of::<u32>() as i32,
                    );
                }
                _ => {}
            }

            if !matches!(fill_rule, fill_rule::FillRule::FillAll) {
                self.handler
                    .stencil_func(web_sys::WebGl2RenderingContext::NOTEQUAL, 0x00, 0xff);
                self.handler.stencil_op_separate(
                    web_sys::WebGl2RenderingContext::FRONT_AND_BACK,
                    web_sys::WebGl2RenderingContext::KEEP,
                    web_sys::WebGl2RenderingContext::KEEP,
                    web_sys::WebGl2RenderingContext::KEEP,
                );
                self.handler.color_mask(true, true, true, true);
            }

            self.handler.draw_elements(
                web_sys::WebGl2RenderingContext::TRIANGLES,
                dst_region.get_count(),
                web_sys::WebGl2RenderingContext::UNSIGNED_INT,
                index_offset * mem::size_of::<u32>() as i32,
            );

            index_offset += dst_region.get_count();
        }

        if !matches!(fill_rule, fill_rule::FillRule::FillAll) {
            self.handler
                .disable(web_sys::WebGl2RenderingContext::STENCIL_TEST);
        }

        Ok(())
    }

    pub(crate) fn compile_shader(
        &self,
        vertex_source: &str,
        fragment_source: &str,
        texels: bool,
        uniforms: &[(String, shader_type::ShaderType)],
    ) -> Result<shader::Shader, ()> {
        let Some(vertex_shader) = self
            .handler
            .create_shader(web_sys::WebGl2RenderingContext::VERTEX_SHADER)
        else {
            return Err(());
        };
        self.handler.shader_source(vertex_shader, vertex_source);
        self.handler.compile_shader(vertex_shader);

        let Some(fragment_shader) = self
            .handler
            .create_shader(web_sys::WebGl2RenderingContext::FRAGMENT_SHADER)
        else {
            self.handler.delete_shader(vertex_shader);
            return Err(());
        };
        self.handler.shader_source(fragment_shader, fragment_source);
        self.handler.compile_shader(fragment_shader);

        let Some(program) = self.handler.create_program() else {
            self.handler.delete_shader(vertex_shader);
            self.handler.delete_shader(fragment_shader);
            return Err(());
        };

        self.handler.attach_shader(program, vertex_shader);
        self.handler.attach_shader(program, fragment_shader);

        for (index, (name, _)) in uniforms.iter().enumerate() {
            self.handler
                .bind_attrib_location(program, index as u32, name.as_str());
        }
        self.handler.link_program(program);

        if !self
            .handler
            .get_program_parameter(program, web_sys::WebGl2RenderingContext::LINK_STATUS)
        {
            let program_info = self.handler.get_program_info_log(program);
            let vertex_info = self.handler.get_shader_info_log(vertex_shader);
            let fragment_info = self.handler.get_shader_info_log(fragment_shader);

            crate::log(&format!(
                "{program_info:?}\n{vertex_info:?}\n{fragment_info:?}"
            ));

            return Err(());
        }

        self.handler.delete_shader(vertex_shader);
        self.handler.delete_shader(fragment_shader);

        Ok(shader::Shader::new(program, uniforms.iter(), texels))
    }

    fn use_program<'a>(
        &self,
        program: ProgramID,
        textures: impl Iterator<Item = (&'a str, TextureID)>,
        uniform_variables: &[uniform_variable::UniformVariable],
    ) {
        if self.program_state.swap_program(program) {
            self.handler.use_program(program);
            self.handler
                .active_texture(web_sys::WebGl2RenderingContext::TEXTURE0);
            self.context.reset_last_texture();
        }

        for uniform in uniform_variables.iter() {
            if uniform.is_empty() {
                continue;
            }
            if !uniform.valid() {
                continue;
            }
            if self
                .program_state
                .last_uniform_eq(uniform.get_name(), uniform.get_value())
            {
                continue;
            }

            if !self.context.uniform(
                &self.handler,
                program,
                uniform.get_name(),
                uniform.get_type(),
                uniform.get_value(),
            ) {
                continue;
            }

            self.program_state
                .add_last_uniform(uniform.get_name(), uniform.get_value());
        }

        let mut offset = 0;
        let mut texture_offset_cache = collections::BTreeMap::<TextureID, i32>::new();
        for (name, texture) in textures {
            if let Some(&cached_offset) = texture_offset_cache.get(&texture) {
                self.context
                    .uniform_int(&self.handler, program, name, cached_offset);
                continue;
            }

            texture_offset_cache.insert(texture, offset);

            if !self
                .context
                .uniform_int(&self.handler, program, name, offset)
            {
                continue;
            }

            if self.program_state.swap_texture_offset(offset) {
                self.handler
                    .active_texture(web_sys::WebGl2RenderingContext::TEXTURE0 + offset as u32);
                self.context.reset_last_texture();
            }
            self.context.bind_texture(&self.handler, texture);

            offset += 1;
        }
    }

    pub(super) fn get_context(&self) -> &graphics_context::GraphicsContext {
        &self.context
    }

    pub(super) fn get_handler(&self) -> &glhandler::GLHandler {
        &self.handler
    }

    pub(super) fn swap_draw_called(&self, value: bool) -> bool {
        self.draw_called.replace(value)
    }
}
