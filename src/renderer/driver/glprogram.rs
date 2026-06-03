use super::{BufferID, ProgramID, VertexArrayID, glhandler};
use std::{cell, collections, mem, sync};

#[derive(Debug, Clone, PartialEq)]
pub struct GLVertexLayoutPart {
    elements_count: usize,
    size: usize,
}

impl GLVertexLayoutPart {
    pub fn new(elements_count: usize) -> Self {
        Self {
            elements_count,
            size: elements_count * mem::size_of::<f32>(),
        }
    }
}

pub trait GLVertexEntity {
    fn layout() -> sync::Arc<[GLVertexLayoutPart]>;

    fn vertex_array(&self, target: &mut [u8]);

    fn index(&self) -> u32;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GLVertexLayout {
    parts: sync::Arc<[GLVertexLayoutPart]>,
    elements_count: usize,
    size: usize,
}

impl GLVertexLayout {
    pub fn new<T>() -> Self
    where
        T: GLVertexEntity,
    {
        Self::new_with_parts(T::layout())
    }

    pub fn new_with_parts(parts: sync::Arc<[GLVertexLayoutPart]>) -> Self {
        let elements_count = parts.iter().fold(0, |acc, part| acc + part.elements_count);

        Self {
            parts,
            elements_count,
            size: elements_count * mem::size_of::<f32>(),
        }
    }

    pub(crate) fn get_elements_count(&self) -> usize {
        self.elements_count
    }

    pub(super) fn enable(&self, hdr: &glhandler::GLHandler) {
        for index in 0..self.parts.len() {
            hdr.enable_vertex_attrib_array(index as u32);
        }

        let mut offset = 0;
        for (index, part) in self.parts.iter().enumerate() {
            hdr.vertex_attrib_pointer(
                index as u32,
                part.elements_count as i32,
                web_sys::WebGl2RenderingContext::FLOAT,
                false,
                self.size as i32,
                offset as i32,
            );
            offset += part.size;
        }
    }

    pub(super) fn disable(&self, hdr: &glhandler::GLHandler) {
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

    layout: Option<GLVertexLayout>,
}

pub(super) struct ProgramState {
    data: cell::RefCell<ProgramStateData>,
}

impl ProgramState {
    pub(super) fn new() -> Self {
        Self {
            data: cell::RefCell::new(ProgramStateData {
                vertex_array: 0,

                array_buffer: 0,
                array_buffer_bytes_size: 0,

                element_array_buffer: 0,
                element_array_buffer_bytes_size: 0,

                last_program: 0,
                last_uniforms: collections::HashMap::new(),
                last_texture_offset: 0,

                layout: None,
            }),
        }
    }

    pub(super) fn reset(&self, hdr: &glhandler::GLHandler) {
        hdr.use_program(0);

        let Ok(mut data) = self.data.try_borrow_mut() else {
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
        let Ok(mut data) = self.data.try_borrow_mut() else {
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
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return false;
        };

        if data.last_texture_offset == offset {
            return false;
        }

        data.last_texture_offset = offset;

        true
    }

    pub(super) fn last_uniform_eq(&self, name: &str, value: &[u8]) -> bool {
        let Ok(data) = self.data.try_borrow() else {
            return false;
        };
        let Some(cached) = data.last_uniforms.get(name) else {
            return false;
        };

        cached.eq(value)
    }

    pub(super) fn add_last_uniform(&self, name: &str, value: &[u8]) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };
        data.last_uniforms.insert(name.to_owned(), value.to_owned());
    }

    pub(super) fn reset_last_uniforms(&self) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
            return;
        };

        data.last_uniforms.clear();
    }

    pub(super) fn set_vertices(
        &self,
        hdr: &glhandler::GLHandler,
        layout: &GLVertexLayout,
        vertices: &[f32],
        indexes: &[u32],
    ) {
        let Ok(mut data) = self.data.try_borrow_mut() else {
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

        let vertices_size = std::mem::size_of_val(vertices);
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

            if let Some(original_layout) = data.layout.as_ref() {
                if original_layout.eq(layout) {
                    original_layout.enable(hdr);
                } else {
                    original_layout.disable(hdr);

                    layout.enable(hdr);
                    data.layout.replace(layout.clone());
                }
            } else {
                layout.enable(hdr);
                data.layout.replace(layout.clone());
            }
        }

        // 适配 element array
        let indexes_size = std::mem::size_of_val(indexes);
        if data.element_array_buffer_bytes_size < indexes_size {
            if data.element_array_buffer != 0 {
                hdr.delete_buffer(data.element_array_buffer);
            }
            let indexes_size = Self::pow2(indexes_size);

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

        hdr.buffer_sub_data(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, 0, unsafe {
            &*std::ptr::slice_from_raw_parts(vertices.as_ptr() as *const u8, vertices_size)
        });
        hdr.buffer_sub_data(
            web_sys::WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            0,
            unsafe {
                &*std::ptr::slice_from_raw_parts(indexes.as_ptr() as *const u8, indexes_size)
            },
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
