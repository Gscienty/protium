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
uniform vec2 source_region_pos;
uniform vec2 source_region_size;
uniform sampler2D source;
out vec4 out_color;
void main() {
    vec4 pos_color = texture(source, frag_source_pos / source_size);
    out_color = pos_color * frag_color;
    // out_color = vec4(1.0, 1.0, 1.0, 1.0);
}"##;

fn create_shader(renderer: &renderer::Renderer) -> renderer::Shader {
    renderer.create_shader(
        NEAREST_FILTER_VERTEX_SHADER_SOURCE.to_owned(),
        NEAREST_FILTER_FRAGMENT_SHADER_SOURCE.to_owned(),
        false,
        renderer::GLVertexLayout::new_with_parts(vec![
            renderer::GLVertexLayoutPart::new(2, 8),
            renderer::GLVertexLayoutPart::new(2, 8),
            renderer::GLVertexLayoutPart::new(4, 16),
        ]),
        vec![
            (
                "projection".to_owned(),
                renderer::ShaderBasicType::Mat4.into(),
            ),
            (
                "source_size".to_owned(),
                renderer::ShaderBasicType::Vec2.into(),
            ),
            (
                "source_region_pos".to_owned(),
                renderer::ShaderBasicType::Vec2.into(),
            ),
            (
                "source_region_size".to_owned(),
                renderer::ShaderBasicType::Vec2.into(),
            ),
        ],
    )
}

static SHADER: sync::OnceLock<renderer::Shader> = sync::OnceLock::new();

pub(crate) fn get_nearest_shader(renderer: &renderer::Renderer) -> &renderer::Shader {
    SHADER.get_or_init(|| create_shader(renderer))
}

pub(crate) fn get_nearest_uniform(
    dst_view: (i32, i32),
    src_view: (i32, i32),
    src_pos: (i32, i32),
    src: (i32, i32),
) -> Vec<u8> {
    [
        // projection
        (2. / dst_view.0 as f32),
        0.,
        0.,
        0.,
        0.,
        (2. / dst_view.1 as f32),
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
        // source_size
        src_view.0 as f32,
        src_view.1 as f32,
        // source region pos
        (src_pos.0 as f32 / src_view.0 as f32),
        (src_pos.1 as f32 / src_view.1 as f32),
        // source region size
        (src.0 as f32 / src_view.0 as f32),
        (src.1 as f32 / src_view.1 as f32),
    ]
    .into_iter()
    .flat_map(|v| v.to_ne_bytes())
    .collect()
}
