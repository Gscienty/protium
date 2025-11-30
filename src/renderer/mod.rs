use std::sync;

use tokio::sync::mpsc;

mod command;
mod command_pool;
mod command_processor;
mod drawing;
mod driver;
mod image;
mod shader;

pub type ImageID = usize;
pub type ShaderID = usize;

pub(crate) struct Renderer {
    next_image_id: sync::atomic::AtomicUsize,
    next_shader_id: sync::atomic::AtomicUsize,

    sender: mpsc::UnboundedSender<command::Command>,

    max_texture_size: i32,
}

impl Renderer {
    pub(crate) fn create(
        canvas: &web_sys::HtmlCanvasElement,
        space: driver::ColorSpace,
        max_vertex_count_limit: usize,
    ) -> Result<Self, ()> {
        let (max_texture_size, sender) =
            command_processor::CommandProcssor::create(canvas, space, max_vertex_count_limit)?;

        Ok(Self {
            next_image_id: sync::atomic::AtomicUsize::new(0),
            next_shader_id: sync::atomic::AtomicUsize::new(0),

            sender,

            max_texture_size,
        })
    }

    pub(crate) fn flush(&self, endframe: bool) {
        let _ = self.sender.send(command::Command::Flush(endframe));
    }

    pub(crate) fn get_max_texture_size(&self) -> i32 {
        self.max_texture_size
    }

    pub(crate) fn create_image(&self, width: i32, height: i32, screen: bool) -> image::Image {
        let id = self
            .next_image_id
            .fetch_add(1, sync::atomic::Ordering::SeqCst);

        let _ = self
            .sender
            .send(command::Command::CreateImage(command::CreateImage::new(
                id, width, height, screen,
            )));
        image::Image::new(id, self.sender.clone(), width, height, screen)
    }

    pub(crate) fn create_shader(
        &self,
        vertex_source: String,
        fragment_source: String,
        texels: bool,
        vertex_layout: driver::GLVertexLayout,
        uniforms: Vec<(String, driver::ShaderType)>,
    ) -> shader::Shader {
        let id = self
            .next_shader_id
            .fetch_add(1, sync::atomic::Ordering::SeqCst);

        let _ = self
            .sender
            .send(command::Command::CreateShader(command::CreateShader::new(
                id,
                vertex_source,
                fragment_source,
                vertex_layout,
                uniforms,
                texels,
            )));
        shader::Shader::new(id, self.sender.clone())
    }
}
