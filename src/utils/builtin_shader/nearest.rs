use std::sync;

use crate::renderer;

const NEAREST_FILTER_VERTEX_SHADER_SOURCE: &str = r##"#version 300 es
precision mediump float;
in vec2 destination_pos;
in vec2 source_pos;
in vec4 color;
uniform mat4 projection;
out vec2 frag_source_pos;
out vec4 frag_color;
void main() {
    gl_Position = projection * vec4(destination_pos, 0.0, 1.0);
    frag_source_pos = source_pos;
    frag_color = color;
}"##;

const NEAREST_FILTER_FRAGMENT_SHADER_SOURCE: &str = r##"#version 300 es
precision mediump float;
in vec2 frag_source_pos;
in vec4 frag_color;
uniform vec2 source_size;
uniform sampler2D source;
out vec4 out_color;
void main() {
    vec2 normalized_coord = frag_source_pos / source_size;
    vec4 pos_color = texture(source, normalized_coord);
    out_color = pos_color * frag_color;
    // out_color = vec4(1.0, 1.0, 1.0, 1.0);
}"##;

fn create_shader(renderer: &renderer::Renderer) -> renderer::Shader {
    renderer.create_shader(
        NEAREST_FILTER_VERTEX_SHADER_SOURCE.to_owned(),
        NEAREST_FILTER_FRAGMENT_SHADER_SOURCE.to_owned(),
        false,
        renderer::GLVertexLayout::new_with_parts(
            [
                renderer::GLVertexLayoutPart::new(2),
                renderer::GLVertexLayoutPart::new(2),
                renderer::GLVertexLayoutPart::new(4),
            ]
            .into(),
        ),
        Box::new(|vertices, typ, (offset_x, offset_y)| {
            for i in (0..vertices.len()).step_by(8) {
                match typ {
                    renderer::LayoutVertexOffset::Destination => {
                        vertices[i] += offset_x;
                        vertices[i + 1] += offset_y;
                    }
                    renderer::LayoutVertexOffset::Source(0) => {
                        vertices[i + 2] += offset_x;
                        vertices[i + 3] += offset_y;
                    }
                    _ => {}
                }
            }
        }),
        [
            (
                "projection".to_owned(),
                renderer::ShaderBasicType::Mat4.into(),
            ),
            (
                "destination_atlas_size".to_owned(),
                renderer::ShaderBasicType::Vec2.into(),
            ),
            (
                "source_size".to_owned(),
                renderer::ShaderBasicType::Vec2.into(),
            ),
        ]
        .into(),
    )
}

static SHADER: sync::OnceLock<renderer::Shader> = sync::OnceLock::new();

pub(crate) fn get_nearest_shader(renderer: &renderer::Renderer) -> &renderer::Shader {
    SHADER.get_or_init(|| create_shader(renderer))
}

#[allow(clippy::too_many_arguments)]
pub fn nearest_quad_vertices_from_dst_src(
    dx0: f32,
    dy0: f32,
    dx1: f32,
    dy1: f32,
    sx0: f32,
    sy0: f32,
    sx1: f32,
    sy1: f32,
    cr: f32,
    cg: f32,
    cb: f32,
    ca: f32,
) -> sync::Arc<[f32]> {
    let dx0 = adjust_detination_pixel(dx0);
    let dy0 = adjust_detination_pixel(dy0);
    let dx1 = adjust_detination_pixel(dx1);
    let dy1 = adjust_detination_pixel(dy1);

    [
        dx0, dy0, sx0, sy0, cr, cg, cb, ca, //
        dx1, dy0, sx1, sy0, cr, cg, cb, ca, //
        dx0, dy1, sx0, sy1, cr, cg, cb, ca, //
        dx1, dy1, sx1, sy1, cr, cg, cb, ca, //
    ]
    .into()
}

#[allow(clippy::too_many_arguments)]
pub fn nearest_quad_vertices_from_src_atlas(
    sx0: f32,
    sy0: f32,
    sx1: f32,
    sy1: f32,
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    tx: f32,
    ty: f32,
    cr: f32,
    cg: f32,
    cb: f32,
    ca: f32,
) -> sync::Arc<[f32]> {
    let x = sx1 - sx0;
    let y = sy1 - sy0;
    let ax = a * x;
    let by = b * y;
    let cx = c * x;
    let dy = d * y;
    let u0 = sx0;
    let v0 = sy0;
    let u1 = sx1;
    let v1 = sy1;

    [
        adjust_detination_pixel(tx),
        adjust_detination_pixel(ty),
        u0,
        v0,
        cr,
        cg,
        cb,
        ca,
        adjust_detination_pixel(ax + tx),
        adjust_detination_pixel(cx + ty),
        u1,
        v0,
        cr,
        cg,
        cb,
        ca,
        adjust_detination_pixel(by + tx),
        adjust_detination_pixel(dy + ty),
        u0,
        v1,
        cr,
        cg,
        cb,
        ca,
        adjust_detination_pixel(ax + by + tx),
        adjust_detination_pixel(cx + dy + ty),
        u1,
        v1,
        cr,
        cg,
        cb,
        ca,
    ]
    .into()
}

fn adjust_detination_pixel(x: f32) -> f32 {
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
