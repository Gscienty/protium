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
        renderer::GLVertexLayout::new_with_parts(vec![renderer::GLVertexLayoutPart::new(2, 8)]),
        vec![(
            "projection".to_owned(),
            renderer::ShaderBasicType::Mat4.into(),
        )],
    )
}

static SHADER: sync::OnceLock<renderer::Shader> = sync::OnceLock::new();

pub(crate) fn get_clear_shader(renderer: &renderer::Renderer) -> &renderer::Shader {
    SHADER.get_or_init(|| create_shader(renderer))
}

pub(crate) fn get_clear_uniform(region: (i32, i32)) -> Vec<u8> {
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
