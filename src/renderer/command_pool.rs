use std::{collections, mem, ptr};

use super::{ImageID, command, driver};

pub(super) struct CommandPool {
    commands: collections::VecDeque<command::Command>,
    write_pixels: collections::BTreeMap<ImageID, Vec<command::WritePixelsArg>>,

    vertices: Vec<u8>,
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
        let indexes = &self.indexes[indexes_offset..indexes_offset + indexes_count];

        graphics.set_vertices(
            layout,
            &self.vertices[vertices_offset..vertices_offset + vertices_count],
            unsafe {
                &*ptr::slice_from_raw_parts(
                    indexes.as_ptr() as *const u8,
                    indexes_count * mem::size_of::<u32>(),
                )
            },
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
        layout: &driver::GLVertexLayout,
        cmd: command::DrawTriangles,
    ) {
        cmd.for_each_sources(|source| {
            self.flush_write_pixels(source.get_image());
        });
        self.flush_write_pixels(cmd.get_destination());

        let vertices_offset = self.vertices.len();
        let indexes_offset = self.indexes.len();
        let appended_vertices_count = cmd.get_vertices_count();
        self.vertices.extend(cmd.get_vertices());

        let need_split =
            self.segment_vertices_count + appended_vertices_count > self.max_vertex_count_limit;

        if !need_split
            && let Some(command::Command::InternalDrawTriangles(last)) = self.commands.back_mut()
            && last.mergeable(&cmd)
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
                command::InternalDrawTriangles::from(cmd, vertices_offset, indexes_offset),
            ));
    }
}
