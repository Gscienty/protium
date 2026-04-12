use crate::{renderer, utils};

use super::{delay, image, material};

#[derive(Debug)]
pub(crate) struct AtlasDrawingQuadDstSrc {
    destination: utils::Rectangle,
    source: utils::Rectangle,

    color: [(f32, f32, f32, f32); 4],
}

impl AtlasDrawingQuadDstSrc {
    pub(crate) fn new(
        destination: utils::Rectangle,
        source: utils::Rectangle,
        color: [(f32, f32, f32, f32); 4],
    ) -> Self {
        Self {
            destination,
            source,
            color,
        }
    }

    fn serialize(&self, destination_offset: utils::Point, source_offset: utils::Point) -> Vec<u8> {
        [
            (self.destination.min.x + destination_offset.x) as f32,
            (self.destination.min.y + destination_offset.y) as f32,
            (self.source.min.x + source_offset.x) as f32,
            (self.source.min.y + source_offset.y) as f32,
            self.color[0].0,
            self.color[0].1,
            self.color[0].2,
            self.color[0].3,
            (self.destination.max.x + destination_offset.x) as f32,
            (self.destination.min.y + destination_offset.y) as f32,
            (self.source.max.x + source_offset.x) as f32,
            (self.source.min.y + source_offset.y) as f32,
            self.color[1].0,
            self.color[1].1,
            self.color[1].2,
            self.color[1].3,
            (self.destination.min.x + destination_offset.x) as f32,
            (self.destination.max.y + destination_offset.y) as f32,
            (self.source.min.x + source_offset.x) as f32,
            (self.source.max.y + source_offset.y) as f32,
            self.color[2].0,
            self.color[2].1,
            self.color[2].2,
            self.color[2].3,
            (self.destination.max.x + destination_offset.x) as f32,
            (self.destination.max.y + destination_offset.y) as f32,
            (self.source.max.x + source_offset.x) as f32,
            (self.source.max.y + source_offset.y) as f32,
            self.color[3].0,
            self.color[3].1,
            self.color[3].2,
            self.color[3].3,
        ]
        .iter()
        .flat_map(|v| v.to_ne_bytes())
        .collect()
    }
}

#[derive(Debug)]
pub(crate) struct AtlasDrawingQuadSrcMaterial {
    source_region: utils::Rectangle,
    arg: (f32, f32, f32, f32),
    t: (f32, f32),
    color: (f32, f32, f32, f32),
}

impl AtlasDrawingQuadSrcMaterial {
    pub(crate) fn new(
        source_region: utils::Rectangle,
        arg: (f32, f32, f32, f32),
        t: (f32, f32),
        color: (f32, f32, f32, f32),
    ) -> Self {
        Self {
            source_region,
            arg,
            t,
            color,
        }
    }

    fn serialize(&self, destination_offset: utils::Point, source_offset: utils::Point) -> Vec<u8> {
        let x = (self.source_region.max.x - self.source_region.min.x) as f32;
        let y = (self.source_region.max.y - self.source_region.min.y) as f32;
        let ax = self.arg.0 * x;
        let by = self.arg.1 * y;
        let cx = self.arg.2 * x;
        let dy = self.arg.3 * y;
        let u0 = self.source_region.min.x as f32 + source_offset.x as f32;
        let v0 = self.source_region.min.y as f32 + source_offset.y as f32;
        let u1 = self.source_region.max.x as f32 + source_offset.x as f32;
        let v1 = self.source_region.max.y as f32 + source_offset.y as f32;

        [
            Self::adjust(self.t.0) + destination_offset.x as f32,
            Self::adjust(self.t.1) + destination_offset.y as f32,
            u0,
            v0,
            self.color.0,
            self.color.1,
            self.color.2,
            self.color.3,
            Self::adjust(ax + self.t.0) + destination_offset.x as f32,
            Self::adjust(cx + self.t.1) + destination_offset.y as f32,
            u1,
            v0,
            self.color.0,
            self.color.1,
            self.color.2,
            self.color.3,
            Self::adjust(by + self.t.0) + destination_offset.x as f32,
            Self::adjust(dy + self.t.1) + destination_offset.y as f32,
            u0,
            v1,
            self.color.0,
            self.color.1,
            self.color.2,
            self.color.3,
            Self::adjust(ax + by + self.t.0) + destination_offset.x as f32,
            Self::adjust(cx + dy + self.t.1) + destination_offset.y as f32,
            u1,
            v1,
            self.color.0,
            self.color.1,
            self.color.2,
            self.color.3,
        ]
        .iter()
        .flat_map(|v| v.to_ne_bytes())
        .collect()
    }

    fn adjust(x: f32) -> f32 {
        let mut x_int = (x as i32) as f32;
        if x < 0. && x != x_int {
            x_int -= 1.;
        }
        let frac = x - x_int;

        if frac < (3. / 16.) {
            x_int
        } else if frac < (8. / 16.) {
            x_int + (5. / 16.)
        } else if frac < (13. / 16.) {
            x_int + (11. / 16.)
        } else {
            x_int + 1.
        }
    }
}

#[derive(Debug)]
enum VertexEntity {
    DstSrc(AtlasDrawingQuadDstSrc),
    SrcMaterial(AtlasDrawingQuadSrcMaterial),
}

#[derive(Debug)]
pub(crate) struct Drawing {
    target: image::Image,
    source: Option<(String, image::Image)>,

    vertex_entity: Option<VertexEntity>,
    destination_atlas_region: utils::Rectangle,
    blend: renderer::Blend,
    fill_rule: renderer::FillRule,
}

impl Drawing {
    pub(super) fn new(target: image::Image, destination_atlas_region: utils::Rectangle) -> Self {
        Self {
            target,

            source: None,
            vertex_entity: None,
            destination_atlas_region,
            blend: renderer::BLEND_DEFAULT,
            fill_rule: Default::default(),
        }
    }

    pub(crate) fn set_source(&mut self, name: &str, image: image::Image) {
        self.source = Some((name.to_owned(), image));
    }

    pub(crate) fn set_blend(&mut self, blend: renderer::Blend) {
        self.blend = blend;
    }

    pub(crate) fn dst_src(&mut self, quad: AtlasDrawingQuadDstSrc) {
        self.vertex_entity = Some(VertexEntity::DstSrc(quad));
    }

    pub(crate) fn src_material(&mut self, quad: AtlasDrawingQuadSrcMaterial) {
        self.vertex_entity = Some(VertexEntity::SrcMaterial(quad));
    }

    pub(crate) fn set_fill_rule(&mut self, fill_rule: renderer::FillRule) {
        self.fill_rule = fill_rule;
    }

    pub(crate) fn draw(self) {
        material::process_material_data(|data, material| {
            if !data.in_frame() {
                data.append_delay(delay::DelayCommand::Draw(self));
                return Option::<()>::None;
            }

            self.execute_draw(data, material);
            Option::<()>::None
        });
    }

    pub(super) fn execute_draw(
        self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
    ) {
        let Some((drawing, destination_atlas_size)) = self.target.process_backend(|backend| {
            let drawing = backend.draw(
                self.destination_atlas_region,
                utils::builtin_shader::get_nearest_shader(material.get_renderer()),
            )?;

            let size = backend.get_image_internal_size();

            Some((drawing, size))
        }) else {
            return;
        };

        drawing.set_blend(self.blend);
        drawing.set_fill_rule(self.fill_rule);

        let Some((source_name, source_image)) = self.source else {
            return;
        };
        let source_atlas_region = source_image.get_atlas_region();

        let Some((source_image, source_size)) = source_image.process_backend(|backend| {
            let image = backend.get_atlas_id()?;
            Some((image, backend.get_image_internal_size()))
        }) else {
            return;
        };

        material_data.add_inframe_source(source_image);

        drawing.set_source(source_name.as_str(), source_image);

        if let Some(vertex_entity) = self.vertex_entity {
            let vertexes = match vertex_entity {
                VertexEntity::DstSrc(dst_src) => {
                    dst_src.serialize(self.destination_atlas_region.min, source_atlas_region.min)
                }
                VertexEntity::SrcMaterial(src_material) => src_material
                    .serialize(self.destination_atlas_region.min, source_atlas_region.min),
            };
            drawing.append_vertexes(vertexes);
            drawing.append_indexes(vec![0, 1, 2, 1, 2, 3]);
        }

        let uniforms =
            utils::builtin_shader::get_nearest_uniform(destination_atlas_size, source_size);
        drawing.set_uniforms(uniforms);

        drawing.draw();
    }
}
