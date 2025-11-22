use std::{collections, str::FromStr, sync};

use wasm_bindgen::JsCast;
use web_sys::{WebGlUniformLocation, js_sys};

use super::{
    BufferID, ColorSpace, FramebufferID, ProgramID, RenderbufferID, ShaderID, TextureID,
    UniformLocationID, VertexArrayID, glentity,
};

pub(super) struct GLHandler {
    glctx: web_sys::WebGl2RenderingContext,

    programs: glentity::GLEntityMap<web_sys::WebGlProgram>,
    shaders: glentity::GLEntityMap<web_sys::WebGlShader>,
    buffers: glentity::GLEntityMap<web_sys::WebGlBuffer>,
    framebuffers: glentity::GLEntityMap<web_sys::WebGlFramebuffer>,
    renderbuffers: glentity::GLEntityMap<web_sys::WebGlRenderbuffer>,
    textures: glentity::GLEntityMap<web_sys::WebGlTexture>,
    vertex_arrays: glentity::GLEntityMap<web_sys::WebGlVertexArrayObject>,
    uniform_locations: sync::RwLock<
        collections::HashMap<ProgramID, glentity::GLEntityMap<web_sys::WebGlUniformLocation>>,
    >,
}

impl GLHandler {
    pub(super) fn new(canvas: &web_sys::HtmlCanvasElement, space: ColorSpace) -> Result<Self, ()> {
        let attr = Self::build_webgl2_context_options()?;

        let Some(glctx) = canvas
            .get_context_with_context_options("webgl2", &attr)
            .map_err(|_| ())?
        else {
            return Err(());
        };
        let glctx = glctx
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .map_err(|_| ())?;

        if !glctx.is_truthy() {
            return Err(());
        }

        match space {
            ColorSpace::Srgb => Self::set_webgl2_context_color_space(&glctx, "srgb")?,
            ColorSpace::DisplayP3 => Self::set_webgl2_context_color_space(&glctx, "display-p3")?,
            _ => {}
        }

        Ok(Self::init(glctx))
    }

    fn build_webgl2_context_options() -> Result<js_sys::Object, ()> {
        let attr = js_sys::Object::new();

        for key in ["alpha", "premultipliedAlpha", "stencil"] {
            js_sys::Reflect::set(
                &attr,
                &js_sys::JsString::from_str(key).map_err(|_| ())?.into(),
                &js_sys::Boolean::from(true),
            )
            .map_err(|_| ())?;
        }

        Ok(attr)
    }

    fn set_webgl2_context_color_space(
        glctx: &web_sys::WebGl2RenderingContext,
        value: &str,
    ) -> Result<(), ()> {
        js_sys::Reflect::set(
            glctx,
            &js_sys::JsString::from_str("drawingBufferColorSpace")
                .map_err(|_| ())?
                .into(),
            &js_sys::JsString::from_str(value).map_err(|_| ())?.into(),
        )
        .map_err(|_| ())?;
        Ok(())
    }

    fn init(glctx: web_sys::WebGl2RenderingContext) -> Self {
        Self {
            glctx,

            programs: glentity::GLEntityMap::new(),
            shaders: glentity::GLEntityMap::new(),
            buffers: glentity::GLEntityMap::new(),
            framebuffers: glentity::GLEntityMap::new(),
            renderbuffers: glentity::GLEntityMap::new(),
            textures: glentity::GLEntityMap::new(),
            vertex_arrays: glentity::GLEntityMap::new(),
            uniform_locations: sync::RwLock::new(collections::HashMap::new()),
        }
    }

    pub(super) fn create_texture(&self) -> Option<TextureID> {
        self.glctx
            .create_texture()
            .map(|value| self.textures.create(value))
    }

    pub(super) fn create_buffer(&self) -> Option<BufferID> {
        self.glctx
            .create_buffer()
            .map(|value| self.buffers.create(value))
    }

    pub(super) fn creae_framebuffer(&self) -> Option<FramebufferID> {
        self.glctx
            .create_framebuffer()
            .map(|value| self.framebuffers.create(value))
    }

    pub(super) fn create_program(&self) -> Option<ProgramID> {
        self.glctx
            .create_program()
            .map(|value| self.programs.create(value))
    }

    pub(super) fn create_renderbuffer(&self) -> Option<RenderbufferID> {
        self.glctx
            .create_renderbuffer()
            .map(|value| self.renderbuffers.create(value))
    }

    pub(super) fn create_shader(&self, type_: u32) -> Option<ShaderID> {
        self.glctx
            .create_shader(type_)
            .map(|value| self.shaders.create(value))
    }

    pub(super) fn create_vertex_array(&self) -> Option<VertexArrayID> {
        self.glctx
            .create_vertex_array()
            .map(|value| self.vertex_arrays.create(value))
    }

    pub(super) fn active_texture(&self, texture: u32) {
        self.glctx.active_texture(texture);
    }

    pub(super) fn attach_shader(&self, program: ProgramID, shader: ShaderID) {
        self.programs.process_entity(program, |program| {
            self.shaders.process_entity(shader, |shader| {
                self.glctx.attach_shader(program, shader);
            });
        });
    }

    pub(super) fn bind_attrib_location(&self, program: ProgramID, index: u32, name: &str) {
        self.programs.process_entity(program, |program| {
            self.glctx.bind_attrib_location(program, index, name);
        });
    }

    pub(super) fn bind_buffer(&self, target: u32, buffer: BufferID) {
        if !self.buffers.process_entity(buffer, |buffer| {
            self.glctx.bind_buffer(target, Some(buffer));
        }) {
            self.glctx.bind_buffer(target, None);
        }
    }

    pub(super) fn bind_framebuffer(&self, target: u32, framebuffer: FramebufferID) {
        if !self
            .framebuffers
            .process_entity(framebuffer, |framebuffer| {
                self.glctx.bind_framebuffer(target, Some(framebuffer));
            })
        {
            self.glctx.bind_framebuffer(target, None);
        }
    }

    pub(super) fn bind_renderbuffer(&self, target: u32, renderbuffer: RenderbufferID) {
        if !self
            .renderbuffers
            .process_entity(renderbuffer, |renderbuffer| {
                self.glctx.bind_renderbuffer(target, Some(renderbuffer));
            })
        {
            self.glctx.bind_renderbuffer(target, None);
        }
    }

    pub(super) fn bind_texture(&self, target: u32, texture: TextureID) {
        if !self.textures.process_entity(texture, |texture| {
            self.glctx.bind_texture(target, Some(texture));
        }) {
            self.glctx.bind_texture(target, None);
        }
    }

    pub(super) fn bind_vertex_array(&self, vertex_array: VertexArrayID) {
        if !self
            .vertex_arrays
            .process_entity(vertex_array, |vertex_array| {
                self.glctx.bind_vertex_array(Some(vertex_array));
            })
        {
            self.glctx.bind_vertex_array(None);
        }
    }

    pub(super) fn blend_equation_separate(&self, mode_rgb: u32, mode_alpha: u32) {
        self.glctx.blend_equation_separate(mode_rgb, mode_alpha);
    }

    pub(super) fn blend_func_separate(
        &self,
        src_rgb: u32,
        dst_rgb: u32,
        src_alpha: u32,
        dst_alpha: u32,
    ) {
        self.glctx
            .blend_func_separate(src_rgb, dst_rgb, src_alpha, dst_alpha);
    }

    pub(super) fn buffer_init(&self, target: u32, size: i32, usage: u32) {
        self.glctx.buffer_data_with_i32(target, size, usage);
    }

    pub(super) fn buffer_sub_data(&self, target: u32, offset: i32, src_data: &[u8]) {
        self.glctx
            .buffer_sub_data_with_i32_and_u8_array(target, offset, src_data);
    }

    pub(super) fn check_framebuffer_status(&self, target: u32) -> u32 {
        self.glctx.check_framebuffer_status(target)
    }

    pub(super) fn clear(&self, mask: u32) {
        self.glctx.clear(mask);
    }

    pub(super) fn color_mask(&self, red: bool, green: bool, blue: bool, alpha: bool) {
        self.glctx.color_mask(red, green, blue, alpha);
    }

    pub(super) fn compile_shader(&self, shader: ShaderID) {
        self.shaders.process_entity(shader, |shader| {
            self.glctx.compile_shader(shader);
        });
    }

    pub(super) fn delete_buffer(&self, buffer: BufferID) {
        let Some(buffer) = self.buffers.remove(buffer) else {
            return;
        };
        self.glctx.delete_buffer(Some(&buffer));
    }

    pub(super) fn delete_framebuffer(&self, framebuffer: FramebufferID) {
        let Some(framebuffer) = self.framebuffers.remove(framebuffer) else {
            return;
        };
        self.glctx.delete_framebuffer(Some(&framebuffer));
    }

    pub(super) fn delete_program(&self, program: ProgramID) {
        let Some(program) = self.programs.remove(program) else {
            return;
        };
        self.glctx.delete_program(Some(&program));
    }

    pub(super) fn delete_renderbuffer(&self, renderbuffer: RenderbufferID) {
        let Some(renderbuffer) = self.renderbuffers.remove(renderbuffer) else {
            return;
        };
        self.glctx.delete_renderbuffer(Some(&renderbuffer));
    }

    pub(super) fn delete_shader(&self, shader: ShaderID) {
        let Some(shader) = self.shaders.remove(shader) else {
            return;
        };
        self.glctx.delete_shader(Some(&shader));
    }

    pub(super) fn delete_texture(&self, texture: TextureID) {
        let Some(texture) = self.textures.remove(texture) else {
            return;
        };
        self.glctx.delete_texture(Some(&texture));
    }

    pub(super) fn delete_vertex_array(&self, vertex_array: VertexArrayID) {
        let Some(vertex_array) = self.vertex_arrays.remove(vertex_array) else {
            return;
        };
        self.glctx.delete_vertex_array(Some(&vertex_array));
    }

    pub(super) fn disable(&self, cap: u32) {
        self.glctx.disable(cap);
    }

    pub(super) fn disable_vertex_attrib_array(&self, index: u32) {
        self.glctx.disable_vertex_attrib_array(index);
    }

    pub(super) fn draw_elements(&self, mode: u32, count: i32, type_: u32, offset: i32) {
        self.glctx
            .draw_elements_with_i32(mode, count, type_, offset);
    }

    pub(super) fn enable(&self, cap: u32) {
        self.glctx.enable(cap);
    }

    pub(super) fn enable_vertex_attrib_array(&self, index: u32) {
        self.glctx.disable_vertex_attrib_array(index);
    }

    pub(super) fn flush(&self) {
        self.glctx.flush();
    }

    pub(super) fn framebuffer_renderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffertarget: u32,
        renderbuffer: RenderbufferID,
    ) {
        if !self
            .renderbuffers
            .process_entity(renderbuffer, |renderbuffer| {
                self.glctx.framebuffer_renderbuffer(
                    target,
                    attachment,
                    renderbuffertarget,
                    Some(renderbuffer),
                );
            })
        {
            self.glctx
                .framebuffer_renderbuffer(target, attachment, renderbuffertarget, None);
        }
    }

    pub(super) fn framebuffer_texture_2d(
        &self,
        target: u32,
        attachment: u32,
        textarget: u32,
        texture: TextureID,
        level: i32,
    ) {
        if !self.textures.process_entity(texture, |texture| {
            self.glctx
                .framebuffer_texture_2d(target, attachment, textarget, Some(texture), level);
        }) {
            self.glctx
                .framebuffer_texture_2d(target, attachment, textarget, None, level);
        }
    }

    pub(super) fn get_uniform_location(
        &self,
        program_id: ProgramID,
        name: &str,
    ) -> Option<UniformLocationID> {
        let mut result = None;
        if !self.programs.process_entity(program_id, |program| {
            let Some(location) = self.glctx.get_uniform_location(program, name) else {
                return;
            };
            let Ok(mut uniform_locations) = self.uniform_locations.write() else {
                return;
            };
            let location_set = uniform_locations
                .entry(program_id)
                .or_insert(glentity::GLEntityMap::new());

            result = Some(location_set.get_or_create(location));
        }) {
            return None;
        }

        result
    }

    pub(super) fn get_integer(&self, pname: u32) -> i32 {
        let Ok(param) = self.glctx.get_parameter(pname) else {
            return 0;
        };

        match pname {
            web_sys::WebGl2RenderingContext::FRAMEBUFFER_BINDING => {
                let Some(id) = self
                    .framebuffers
                    .get_entity_id(param.as_ref().unchecked_ref())
                else {
                    return 0;
                };

                id as i32
            }
            web_sys::WebGl2RenderingContext::MAX_TEXTURE_SIZE => {
                let Some(id) = param.as_f64() else {
                    return 0;
                };
                id as i32
            }
            _ => panic!("unexpected pname"),
        }
    }

    pub(super) fn is_program(&self, program: ProgramID) -> bool {
        let mut result = false;
        if !self.programs.process_entity(program, |program| {
            result = self.glctx.is_program(Some(program));
        }) {
            return false;
        }

        result
    }

    pub(super) fn link_program(&self, program: ProgramID) {
        self.programs.process_entity(program, |program| {
            self.glctx.link_program(program);
        });
    }

    pub(super) fn pixel_store(&self, pname: u32, param: i32) {
        self.glctx.pixel_storei(pname, param);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn read_pixels(
        &self,
        dst_data: &mut [u8],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        type_: u32,
    ) {
        let _ = self.glctx.read_pixels_with_u8_array_and_dst_offset(
            x, y, width, height, format, type_, dst_data, 0,
        );
    }

    pub(super) fn renderbuffer_storage(
        &self,
        target: u32,
        internalformat: u32,
        width: i32,
        height: i32,
    ) {
        self.glctx
            .renderbuffer_storage(target, internalformat, width, height);
    }

    pub(super) fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.glctx.scissor(x, y, width, height);
    }

    pub(super) fn stencil_func(&self, func: u32, ref_: i32, mask: u32) {
        self.glctx.stencil_func(func, ref_, mask);
    }

    pub(super) fn shader_source(&self, shader: ShaderID, source: &str) {
        self.shaders.process_entity(shader, |shader| {
            self.glctx.shader_source(shader, source);
        });
    }

    pub(super) fn stencil_op_separate(&self, face: u32, fail: u32, zfail: u32, zpass: u32) {
        self.glctx.stencil_op_separate(face, fail, zfail, zpass);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        format: u32,
        type_: u32,
        pixels: Option<&[u8]>,
    ) {
        let _ = self
            .glctx
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                target,
                level,
                internalformat,
                width,
                height,
                0,
                format,
                type_,
                pixels,
            );
    }

    pub(super) fn tex_parameter(&self, target: u32, pname: u32, param: i32) {
        self.glctx.tex_parameteri(target, pname, param);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: u32,
        type_: u32,
        pixels: &[u8],
    ) {
        let _ = self
            .glctx
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                target,
                level,
                xoffset,
                yoffset,
                width,
                height,
                format,
                type_,
                Some(pixels),
            );
    }

    pub(super) fn uniform_1i(&self, program: ProgramID, location: UniformLocationID, data: i32) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform1i(Some(location), data);
        });
    }

    pub(super) fn uniform_1iv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[i32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform1iv_with_i32_array(Some(location), data);
        });
    }

    pub(super) fn uniform_1fv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[f32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform1fv_with_f32_array(Some(location), data);
        });
    }

    pub(super) fn uniform_2fv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[f32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform2fv_with_f32_array(Some(location), data);
        });
    }

    pub(super) fn uniform_2iv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[i32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform2iv_with_i32_array(Some(location), data);
        });
    }

    pub(super) fn uniform_3fv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[f32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform3fv_with_f32_array(Some(location), data);
        });
    }

    pub(super) fn uniform_3iv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[i32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform3iv_with_i32_array(Some(location), data);
        });
    }

    pub(super) fn uniform_4fv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[f32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform4fv_with_f32_array(Some(location), data);
        });
    }

    pub(super) fn uniform_4iv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[i32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx.uniform4iv_with_i32_array(Some(location), data);
        });
    }

    pub(super) fn uniform_matrix_2fv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[f32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx
                .uniform_matrix2fv_with_f32_array(Some(location), false, data);
        });
    }

    pub(super) fn uniform_matrix_3fv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[f32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx
                .uniform_matrix3fv_with_f32_array(Some(location), false, data);
        });
    }

    pub(super) fn uniform_matrix_4fv(
        &self,
        program: ProgramID,
        location: UniformLocationID,
        data: &[f32],
    ) {
        self.process_uniform_location(program, location, |location| {
            self.glctx
                .uniform_matrix4fv_with_f32_array(Some(location), false, data);
        });
    }

    fn process_uniform_location<F>(&self, program: ProgramID, location: UniformLocationID, cb: F)
    where
        F: FnOnce(&WebGlUniformLocation),
    {
        let Ok(data) = self.uniform_locations.read() else {
            return;
        };
        let Some(uniform_locations) = data.get(&program) else {
            return;
        };
        uniform_locations.process_entity(location, cb);
    }

    pub(super) fn use_program(&self, program: ProgramID) {
        if !self.programs.process_entity(program, |program| {
            self.glctx.use_program(Some(program));
        }) {
            self.glctx.use_program(None);
        }
    }

    pub(super) fn vertex_attrib_pointer(
        &self,
        index: u32,
        size: i32,
        type_: u32,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        self.glctx
            .vertex_attrib_pointer_with_i32(index, size, type_, normalized, stride, offset);
    }

    pub(super) fn viewpoint(&self, x: i32, y: i32, width: i32, height: i32) {
        self.glctx.viewport(x, y, width, height);
    }

    pub(super) fn get_program_parameter(&self, program: ProgramID, pname: u32) -> bool {
        let mut result = false;
        if !self.programs.process_entity(program, |program| {
            result = self
                .glctx
                .get_program_parameter(program, pname)
                .as_bool()
                .unwrap_or(false);
        }) {
            return false;
        }

        result
    }
}
