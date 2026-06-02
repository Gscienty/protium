use std::sync;

use crate::renderer;

const CLEAR_VERTEX_SHADER_SOURCE: &str = r##"#version 300 es
precision mediump float;
in vec2 destination_pos;
uniform mat4 projection;
void main() {
    gl_Position = projection * vec4(destination_pos, 0.0, 1.0);
}"##;

const CLEAR_FRAGMENT_SHADER_SOURCE: &str = r##"#version 300 es
precision mediump float;
out vec4 out_color;
void main() {
    out_color = vec4(0.0, 0.0, 0.0, 0.0);
}"##;

fn create_shader(renderer: &renderer::Renderer) -> renderer::Shader {
    renderer.create_shader(
        CLEAR_VERTEX_SHADER_SOURCE.to_owned(),
        CLEAR_FRAGMENT_SHADER_SOURCE.to_owned(),
        false,
        renderer::GLVertexLayout::new_with_parts([renderer::GLVertexLayoutPart::new(2, 8)].into()),
        Box::new(|vertices, _, (offset_x, offset_y)| {
            for i in (0..vertices.len()).step_by(2) {
                vertices[i] += offset_x;
                vertices[i + 1] += offset_y;
            }
        }),
        [(
            "projection".to_owned(),
            renderer::ShaderBasicType::Mat4.into(),
        )]
        .into(),
    )
}

static SHADER: sync::OnceLock<renderer::Shader> = sync::OnceLock::new();

pub(crate) fn get_clear_shader(renderer: &renderer::Renderer) -> &renderer::Shader {
    SHADER.get_or_init(|| create_shader(renderer))
}

pub(crate) fn get_clear_uniform(region: (i32, i32)) -> sync::Arc<[u8]> {
    vec![
        (2. / region.0 as f32),
        0.,
        0.,
        0.,
        0.,
        (2. / region.1 as f32),
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
    ]
    .into_iter()
    .flat_map(|v| v.to_ne_bytes())
    .collect()
}

pub fn clear_quad_vertices_from_dst_src(
    dx0: f32,
    dy0: f32,
    dx1: f32,
    dy1: f32,
) -> sync::Arc<[f32]> {
    let dx0 = adjust_detination_pixel(dx0);
    let dy0 = adjust_detination_pixel(dy0);
    let dx1 = adjust_detination_pixel(dx1);
    let dy1 = adjust_detination_pixel(dy1);

    [
        dx0, dy0, //
        dx1, dy0, //
        dx0, dy1, //
        dx1, dy1, //
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
