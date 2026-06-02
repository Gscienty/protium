use std::{collections, mem, ptr, sync};

use crate::renderer::command::DrawSourceArg;

use super::{ImageID, command, driver};

pub(super) struct CommandPool {
    commands: collections::VecDeque<command::Command>,
    write_pixels: collections::BTreeMap<ImageID, Vec<command::WritePixelsArg>>,

    vertices: Vec<f32>,
    indexes: Vec<u32>,
    segment_vertices_count: usize,

    max_vertex_count_limit: usize,
}

impl CommandPool {
    pub(super) fn new(max_vertex_count_limit: usize) -> Self {
        Self {
            commands: collections::VecDeque::new(),
            write_pixels: collections::BTreeMap::new(),

            vertices: Vec::new(),
            indexes: Vec::new(),
            segment_vertices_count: 0,
            max_vertex_count_limit,
        }
    }

    pub(super) fn clear(&mut self) {
        self.vertices.clear();
        self.indexes.clear();

        self.segment_vertices_count = 0;
    }

    pub(super) fn dequeue(&mut self) -> Option<command::Command> {
        self.commands.pop_front()
    }

    pub(super) fn enqueue(&mut self, cmd: command::Command) {
        self.commands.push_back(cmd);
    }

    pub(super) fn set_vertices(
        &self,
        layout: &driver::GLVertexLayout,
        graphics: &driver::Graphics,
        vertices_offset: usize,
        vertices_count: usize,
        indexes_offset: usize,
        indexes_count: usize,
    ) {
        graphics.set_vertices(
            layout,
            &self.vertices[vertices_offset..vertices_offset + vertices_count],
            &self.indexes[indexes_offset..indexes_offset + indexes_count],
        );
    }

    pub(super) fn enqueue_write_pixels(&mut self, cmd: command::WritePixels) {
        let records = self.write_pixels.entry(cmd.get_destination()).or_default();

        let mut pointer = 0;
        let records_len = records.len();
        for offset in 0..records_len {
            let arg = &mut records[offset];
            if arg.get_region().is_in(cmd.get_arg().get_region()) {
                arg.clear_pixels();
                continue;
            }

            if pointer != offset {
                records.swap(pointer, offset);
            }
            pointer += 1;
        }

        records.truncate(pointer);
        records.push(cmd.into());
    }

    pub(super) fn flush_write_pixels(&mut self, destination: ImageID) {
        let Some(args) = self.write_pixels.remove(&destination) else {
            return;
        };

        self.enqueue(command::Command::InternalWritePixels(
            command::InternalWritePixels::new(destination, args),
        ));
    }

    pub(super) fn dispose_write_image(&mut self, destination: ImageID) {
        self.write_pixels.remove(&destination);
    }

    fn append_indexes(&mut self, indexes: &[u32], offset: u32) {
        self.indexes
            .extend(indexes.iter().map(|index| index + offset));
    }

    pub(super) fn enqueue_draw_triangles(
        &mut self,
        images: &collections::BTreeMap<ImageID, driver::Image>,
        layout: &driver::GLVertexLayout,
        cmd: command::Draw,
    ) {
        self.flush_write_pixels(cmd.get_destination());

        let vertices_offset = self.vertices.len();
        let indexes_offset = self.indexes.len();
        let appended_vertices_count = cmd.get_vertices_count();
        self.vertices.extend(cmd.get_vertices());

        let uniforms = self.prepend_uniforms(
            images,
            cmd.get_destination(),
            cmd.get_sources(),
            cmd.get_uniforms(),
        );

        let need_split =
            self.segment_vertices_count + appended_vertices_count > self.max_vertex_count_limit;

        if !need_split
            && let Some(command::Command::InternalDrawTriangles(last)) = self.commands.back_mut()
            && last.mergeable(&self.vertices, &cmd, &uniforms, layout.get_size())
        {
            last.merge(&cmd);

            self.append_indexes(
                cmd.get_indexes(),
                (self.segment_vertices_count / layout.get_size()) as u32,
            );
            self.segment_vertices_count += appended_vertices_count;

            return;
        }

        self.append_indexes(cmd.get_indexes(), 0);
        self.segment_vertices_count = appended_vertices_count;
        self.commands
            .push_back(command::Command::InternalDrawTriangles(
                command::InternalDrawTriangles::from(
                    cmd,
                    uniforms,
                    vertices_offset,
                    indexes_offset,
                ),
            ));
    }

    fn prepend_uniforms(
        &self,
        images: &collections::BTreeMap<ImageID, driver::Image>,
        destination: ImageID,
        sources: &[DrawSourceArg],
        uniforms: &[u8],
    ) -> sync::Arc<[u8]> {
        let mut prepend_uniforms = Vec::with_capacity(16 + sources.len() * 2);
        if let Some(destination) = images.get(&destination) {
            let (w, h) = destination.viewport_size();
            prepend_uniforms.extend([
                (2. / w as f32),
                0.,
                0.,
                0.,
                0.,
                (2. / h as f32),
                0.,
                0.,
                0.,
                0.,
                1.,
                0.,
                -1.,
                -1.,
                0.,
                1.,
            ])
        } else {
            prepend_uniforms.extend([0.; 16]);
        }
        for source in sources.iter() {
            let Some(source) = images.get(&source.get_image()) else {
                prepend_uniforms.extend([0.; 2]);
                continue;
            };
            let (w, h) = source.viewport_size();
            prepend_uniforms.extend([w as f32, h as f32]);
        }

        if uniforms.is_empty() {
            return prepend_uniforms
                .into_iter()
                .flat_map(|v| v.to_ne_bytes())
                .collect();
        }

        let prepend_uniforms_count = prepend_uniforms.len() * mem::size_of::<f32>();
        let uniforms_count = uniforms.len();
        let mut final_uniforms = vec![0u8; prepend_uniforms_count + uniforms_count];

        unsafe {
            ptr::copy_nonoverlapping(
                prepend_uniforms.as_ptr() as *const u8,
                final_uniforms.as_mut_ptr(),
                prepend_uniforms_count,
            );
            ptr::copy_nonoverlapping(
                uniforms.as_ptr(),
                final_uniforms.as_mut_ptr().add(prepend_uniforms_count),
                uniforms_count,
            );
        }

        final_uniforms.into()
    }
}
