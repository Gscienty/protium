use super::{BufferID, ProgramID, VertexArrayID, glhandler};
use std::{collections, marker, mem, sync};

pub struct GLVertexLayoutPart {
    name: String,
    elems_count: usize,
}

impl GLVertexLayoutPart {
    pub fn new(name: &str, elems_count: usize) -> Self {
        Self {
            name: name.to_owned(),
            elems_count,
        }
    }
}

pub trait GLVertexEntity {
    fn layout() -> Vec<GLVertexLayoutPart>;

    fn vertex_array(&self, target: &mut [u8]);

    fn index(&self) -> u32;
}

pub(super) struct GLVertexLayout<T>
where
    T: GLVertexEntity,
{
    parts: Vec<GLVertexLayoutPart>,
    elems_count_total: usize,

    _entity: marker::PhantomData<T>,
}

impl<T> GLVertexLayout<T>
where
    T: GLVertexEntity,
{
    pub fn new() -> Self {
        let parts = T::layout();
        let elems_count_total = parts.iter().fold(0, |acc, part| acc + part.elems_count);

        Self {
            parts,
            elems_count_total,

            _entity: Default::default(),
        }
    }

    pub fn enable(&self, hdr: &glhandler::GLHandler) {
        for index in 0..self.parts.len() {
            hdr.enable_vertex_attrib_array(index as u32);
        }

        let mut offset = 0;
        for (index, part) in self.parts.iter().enumerate() {
            hdr.vertex_attrib_pointer(
                index as u32,
                part.elems_count as i32,
                web_sys::WebGl2RenderingContext::FLOAT,
                false,
                (mem::size_of::<f32>() * self.elems_count_total) as i32,
                offset as i32,
            );
            offset += mem::size_of::<f32>() * part.elems_count;
        }
    }

    pub fn disable(&self, hdr: &glhandler::GLHandler) {
        for index in (0..self.parts.len()).rev() {
            hdr.disable_vertex_attrib_array(index as u32);
        }
    }
}

struct ProgramStateData {
    vertex_array: VertexArrayID,

    array_buffer: BufferID,
    array_buffer_bytes_size: usize,

    element_array_buffer: BufferID,
    element_array_buffer_bytes_size: usize,

    last_program: ProgramID,
    last_uniforms: collections::HashMap<String, Vec<u8>>,
    last_texture_offset: i32,
}

pub(super) struct ProgramState<T>
where
    T: GLVertexEntity,
{
    data: sync::RwLock<ProgramStateData>,

    layout: GLVertexLayout<T>,
}

impl<T> ProgramState<T>
where
    T: GLVertexEntity,
{
    pub(super) fn new() -> Self {
        Self {
            data: sync::RwLock::new(ProgramStateData {
                vertex_array: 0,

                array_buffer: 0,
                array_buffer_bytes_size: 0,

                element_array_buffer: 0,
                element_array_buffer_bytes_size: 0,

                last_program: 0,
                last_uniforms: collections::HashMap::new(),
                last_texture_offset: 0,
            }),
            layout: GLVertexLayout::<T>::new(),
        }
    }

    pub(super) fn reset(&self, hdr: &glhandler::GLHandler) {
        hdr.use_program(0);

        let Ok(mut data) = self.data.write() else {
            return;
        };

        data.vertex_array = 0;

        data.array_buffer = 0;
        data.array_buffer_bytes_size = 0;

        data.element_array_buffer = 0;
        data.element_array_buffer_bytes_size = 0;

        data.last_program = 0;
        data.last_uniforms.clear();
    }

    pub(super) fn swap_program(&self, program: ProgramID) -> bool {
        let Ok(mut data) = self.data.write() else {
            return false;
        };

        if data.last_program == program {
            return false;
        }
        data.last_program = program;
        data.last_uniforms.clear();
        data.last_texture_offset = 0;

        true
    }

    pub(super) fn swap_texture_offset(&self, offset: i32) -> bool {
        let Ok(mut data) = self.data.write() else {
            return false;
        };

        if data.last_texture_offset == offset {
            return false;
        }

        data.last_texture_offset = offset;

        true
    }

    pub(super) fn last_uniform_eq(&self, name: &str, value: &[u8]) -> bool {
        let Ok(data) = self.data.read() else {
            return false;
        };
        let Some(cached) = data.last_uniforms.get(name) else {
            return false;
        };

        cached.eq(value)
    }

    pub(super) fn add_last_uniform(&self, name: &str, value: &[u8]) {
        let Ok(mut data) = self.data.write() else {
            return;
        };
        data.last_uniforms.insert(name.to_owned(), value.to_owned());
    }

    pub(super) fn reset_last_uniforms(&self) {
        let Ok(mut data) = self.data.write() else {
            return;
        };

        data.last_uniforms.clear();
    }

    pub(super) fn set_vertices(&self, hdr: &glhandler::GLHandler, entities: &[T]) {
        let entity_vertex_bytes_size = self.layout.elems_count_total * mem::size_of::<f32>();
        let vertices_size = entity_vertex_bytes_size * entities.len();
        let elements_size = mem::size_of::<u32>() * entities.len();

        // 将 entities 序列化
        let mut buffer = vec![0u8; vertices_size];
        let mut element_buffer = vec![0u8; elements_size];

        let mut vertex_offset = 0;
        let mut element_offset = 0;
        for vertex in entities.iter() {
            element_buffer[element_offset..element_offset + mem::size_of::<u32>()]
                .copy_from_slice(&vertex.index().to_ne_bytes());
            vertex.vertex_array(&mut buffer[vertex_offset..]);

            element_offset += mem::size_of::<u32>();
            vertex_offset += entity_vertex_bytes_size;
        }

        let Ok(mut data) = self.data.write() else {
            return;
        };

        // 适配 vertex array
        if data.vertex_array == 0 {
            let Some(vertex_array) = hdr.create_vertex_array() else {
                return;
            };
            data.vertex_array = vertex_array;
        }
        hdr.bind_vertex_array(data.vertex_array);

        if data.array_buffer_bytes_size < vertices_size {
            if data.array_buffer != 0 {
                hdr.delete_buffer(data.array_buffer);
            }

            let vertices_size = Self::pow2(vertices_size);

            let Some(array_buffer) = hdr.create_buffer() else {
                return;
            };
            hdr.bind_buffer(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, array_buffer);
            hdr.buffer_init(
                web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
                vertices_size as i32,
                web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
            );
            data.array_buffer = array_buffer;
            data.array_buffer_bytes_size = vertices_size;

            self.layout.enable(hdr);
        }

        // 适配 element array
        if data.element_array_buffer_bytes_size < elements_size {
            if data.element_array_buffer != 0 {
                hdr.delete_buffer(data.element_array_buffer);
            }

            let indexes_size = Self::pow2(elements_size);

            let Some(element_array_buffer) = hdr.create_buffer() else {
                return;
            };
            hdr.bind_buffer(
                web_sys::WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                element_array_buffer,
            );
            hdr.buffer_init(
                web_sys::WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                indexes_size as i32,
                web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
            );
            data.element_array_buffer = element_array_buffer;
            data.element_array_buffer_bytes_size = indexes_size;
        }

        hdr.buffer_sub_data(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            0,
            buffer.as_ref(),
        );
        hdr.buffer_sub_data(
            web_sys::WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            0,
            element_buffer.as_ref(),
        );
    }

    fn pow2(x: usize) -> usize {
        if x > (i32::MAX as usize).div_ceil(2) {
            return i32::MAX as usize;
        }

        let mut p2 = 1;
        while p2 < x {
            p2 *= 2;
        }
        p2
    }
}
