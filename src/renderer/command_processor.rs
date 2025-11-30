use std::{collections, convert};

use tokio::sync::mpsc;

use super::{ImageID, ShaderID, command, command_pool, driver};

pub(super) struct CommandProcssor {
    graphics: driver::Graphics,

    images: collections::BTreeMap<ImageID, driver::Image>,
    shaders: collections::BTreeMap<ShaderID, (driver::Shader, driver::GLVertexLayout)>,

    command_pool: command_pool::CommandPool,

    receiver: mpsc::UnboundedReceiver<command::Command>,
}

impl CommandProcssor {
    pub(crate) fn create(
        canvas: &web_sys::HtmlCanvasElement,
        space: driver::ColorSpace,
        max_vertex_count_limit: usize,
    ) -> Result<(i32, mpsc::UnboundedSender<command::Command>), ()> {
        let (sender, receiver) = mpsc::unbounded_channel();

        let graphics = driver::Graphics::new(canvas, space)?;
        graphics.initialize();
        let max_texture_size = graphics.get_max_texture_size();

        tokio::spawn(async move {
            Self::new(graphics, receiver, max_vertex_count_limit)
                .run()
                .await;
        });

        Ok((max_texture_size, sender))
    }

    fn new(
        graphics: driver::Graphics,
        receiver: mpsc::UnboundedReceiver<command::Command>,
        max_vertex_count_limit: usize,
    ) -> Self {
        Self {
            graphics,

            images: collections::BTreeMap::new(),
            shaders: collections::BTreeMap::new(),

            command_pool: command_pool::CommandPool::new(max_vertex_count_limit),

            receiver,
        }
    }

    async fn run(&mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                command::Command::DrawTriangles(cmd) => {
                    let Some((_, layout)) = self.shaders.get(&cmd.get_shader()) else {
                        continue;
                    };
                    self.command_pool.enqueue_draw_triangles(layout, cmd);
                }
                command::Command::WritexPixels(cmd) => {
                    self.command_pool.enqueue_write_pixels(cmd);
                }
                command::Command::FlushWritePixels(destination) => {
                    self.command_pool.flush_write_pixels(destination);
                }
                command::Command::Flush(endframe) => {
                    self.flush(endframe);
                }
                command::Command::CreateImage(cmd) => {
                    self.create_image(cmd);
                }
                command::Command::CreateShader(cmd) => {
                    self.create_shader(cmd);
                }
                _ => self.command_pool.enqueue(cmd),
            }
        }
    }

    fn flush(&mut self, endframe: bool) {
        while let Some(cmd) = self.command_pool.dequeue() {
            match cmd {
                command::Command::InternalDrawTriangles(cmd) => {
                    self.draw_triangles(cmd);
                }
                command::Command::InternalWritePixels(cmd) => {
                    self.write_pixels(cmd);
                }
                command::Command::DisposeImage(image) => {
                    self.dispose_image(image);
                }
                command::Command::DisposeShader(shader) => {
                    self.dispose_shader(shader);
                }
                command::Command::ReadPixels(cmd) => {
                    self.read_pixels(cmd);
                }

                _ => {}
            }
        }
        self.graphics.flush(endframe);
        self.command_pool.clear();
    }

    fn create_image(&mut self, cmd: command::CreateImage) {
        let image = if cmd.get_is_screen() {
            self.graphics
                .new_screen_image(cmd.get_width(), cmd.get_height())
        } else {
            let Ok(image) = self.graphics.new_image(cmd.get_width(), cmd.get_height()) else {
                return;
            };
            image
        };

        self.images.insert(cmd.get_id(), image);
    }

    fn create_shader(&mut self, cmd: command::CreateShader) {
        let Ok(shader) = self.graphics.compile_shader(
            cmd.get_vertex_source(),
            cmd.get_fragment_source(),
            cmd.get_texels(),
            cmd.get_uniforms(),
        ) else {
            return;
        };

        self.shaders.insert(cmd.get_id(), (shader, cmd.into()));
    }

    fn dispose_image(&mut self, image: ImageID) {
        self.command_pool.dispose_write_image(image);
        if let Some(image) = self.images.remove(&image) {
            image.dispose(&self.graphics);
        };
    }

    fn dispose_shader(&mut self, shader: ShaderID) {
        let Some((shader, _)) = self.shaders.remove(&shader) else {
            return;
        };
        shader.dispose(&self.graphics);
    }

    fn read_pixels(&mut self, cmd: command::ReadPixels) {
        let Some(image) = self.images.get(&cmd.get_id()) else {
            cmd.send(vec![]);
            return;
        };

        let Ok(result) = image.read_pixels(&self.graphics, cmd.get_regions().iter()) else {
            cmd.send(vec![]);
            return;
        };

        cmd.send(result);
    }

    fn write_pixels(&mut self, cmd: command::InternalWritePixels) {
        let Some(image) = self.images.get(&cmd.get_destination()) else {
            return;
        };

        let _ = image.write_pixels(
            &self.graphics,
            convert::Into::<Vec<command::WritePixelsArg>>::into(cmd)
                .into_iter()
                .map(convert::Into::<driver::PixelsArgs>::into),
        );
    }

    fn draw_triangles(&mut self, cmd: command::InternalDrawTriangles) {
        if cmd.destination_regions_is_empty() {
            return;
        }

        let Some(destination) = self.images.get(&cmd.get_destination()) else {
            return;
        };
        let Some((shader, layout)) = self.shaders.get(&cmd.get_shader()) else {
            return;
        };

        self.command_pool.set_vertices(
            layout,
            &self.graphics,
            cmd.get_vertices_offset(),
            cmd.get_vertices_count(),
            cmd.get_indexes_offset(),
            cmd.get_indexes_count(),
        );

        let _ = self.graphics.draw_triangles(
            destination,
            cmd.get_sources().iter().map(|source| {
                let Some(image) = self.images.get(&source.get_image()) else {
                    return (source.get_name(), 0);
                };
                (source.get_name(), image.get_texture())
            }),
            shader,
            cmd.get_destination_regions().iter(),
            cmd.get_blend(),
            cmd.get_fill_rule(),
            cmd.get_uniforms(),
        );
    }
}
