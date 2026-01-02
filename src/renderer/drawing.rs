use std::cell;

use tokio::sync::mpsc;

use crate::utils;

use super::{ImageID, ShaderID, command, driver};

#[derive(Debug)]
struct DrawingData {
    destination: ImageID,
    sources: Vec<command::DrawTriangleSourceArg>,
    vertices: Vec<u8>,
    indexes: Vec<u32>,
    blend: driver::Blend,
    destination_region: utils::Rectangle,
    shader: ShaderID,
    uniforms: Vec<u8>,
    fill_rule: driver::FillRule,
}

#[derive(Debug)]
pub(crate) struct Drawing {
    data: cell::RefCell<DrawingData>,
    sender: mpsc::UnboundedSender<command::Command>,
}

impl Drawing {
    pub(super) fn new(
        destination: ImageID,
        destination_region: utils::Rectangle,
        shader: ShaderID,
        sender: mpsc::UnboundedSender<command::Command>,
    ) -> Self {
        Self {
            data: cell::RefCell::new(DrawingData {
                destination,
                sources: Vec::new(),
                vertices: Vec::new(),
                indexes: Vec::new(),
                blend: driver::BLEND_DEFAULT,
                destination_region,
                shader,
                uniforms: Vec::new(),
                fill_rule: driver::FillRule::FillAll,
            }),
            sender,
        }
    }

    pub(crate) fn append_vertexes(&self, vertices: Vec<u8>) -> &Self {
        let mut data = self.data.borrow_mut();
        data.vertices.extend(vertices);

        self
    }

    pub(crate) fn append_indexes(&self, indexes: Vec<u32>) -> &Self {
        let mut data = self.data.borrow_mut();
        data.indexes.extend(indexes);

        self
    }

    pub(crate) fn set_source(&self, name: &str, image: ImageID) -> &Self {
        let mut data = self.data.borrow_mut();
        data.sources
            .push(command::DrawTriangleSourceArg::new(name.to_owned(), image));

        self
    }

    pub(crate) fn set_uniforms(&self, uniforms: Vec<u8>) -> &Self {
        let mut data = self.data.borrow_mut();
        data.uniforms = uniforms;

        self
    }

    pub(crate) fn set_blend(&self, blend: driver::Blend) -> &Self {
        let mut data = self.data.borrow_mut();
        data.blend = blend;

        self
    }

    pub(crate) fn set_fill_rule(&self, fill_rule: driver::FillRule) -> &Self {
        let mut data = self.data.borrow_mut();
        data.fill_rule = fill_rule;

        self
    }

    pub(crate) fn draw(&self) {
        let data = self.data.replace(DrawingData {
            destination: 0,
            sources: Default::default(),
            vertices: Default::default(),
            indexes: Default::default(),
            blend: driver::BLEND_DEFAULT,
            destination_region: utils::Rectangle::new(0, 0, 0, 0),
            shader: 0,
            uniforms: Default::default(),
            fill_rule: driver::FillRule::FillAll,
        });

        let _ = self.sender.send(command::Command::DrawTriangles(
            command::DrawTriangles::new(
                data.destination,
                data.sources,
                data.vertices,
                data.indexes,
                data.blend,
                data.destination_region,
                data.shader,
                data.uniforms,
                data.fill_rule,
            ),
        ));
    }
}
