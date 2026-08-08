use std::sync::Arc;

use vulkano::device::Device;
use vulkano::shader::ShaderModule;

pub struct ShaderBundle {
    pub vertex: Arc<ShaderModule>,
    pub fragment: Arc<ShaderModule>,
}

pub struct TextShaderBundle {
    pub vertex: Arc<ShaderModule>,
    pub fragment: Arc<ShaderModule>,
}

pub fn load(device: Arc<Device>) -> Result<ShaderBundle, Box<dyn std::error::Error>> {
    let vertex = vertex_shader::load(device.clone())?;
    let fragment = fragment_shader::load(device.clone())?;
    Ok(ShaderBundle { vertex, fragment })
}

pub fn load_text_shaders(
    device: Arc<Device>,
) -> Result<TextShaderBundle, Box<dyn std::error::Error>> {
    let vertex = text_vertex_shader::load(device.clone())?;
    let fragment = text_fragment_shader::load(device.clone())?;
    Ok(TextShaderBundle { vertex, fragment })
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r#"
            #version 450
            layout(location = 0) in vec2 position;
            layout(location = 1) in vec4 color;
            layout(push_constant, std430) uniform PushConstants {
                vec2 scale;
                vec2 offset;
            } pc;
            layout(location = 0) out vec4 v_color;
            void main() {
                vec2 ndc = position * pc.scale + pc.offset;
                gl_Position = vec4(ndc, 0.0, 1.0);
                v_color = color;
            }
        "#,
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r#"
            #version 450
            layout(location = 0) in vec4 v_color;
            layout(location = 0) out vec4 f_color;
            void main() {
                f_color = v_color;
            }
        "#,
    }
}

mod text_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r#"
            #version 450
            layout(location = 0) in vec2 position;
            layout(location = 1) in vec2 uv;
            layout(location = 2) in vec4 color;
            layout(push_constant, std430) uniform PushConstants {
                vec2 scale;
                vec2 offset;
            } pc;
            layout(location = 0) out vec2 v_uv;
            layout(location = 1) out vec4 v_color;
            void main() {
                gl_Position = vec4(position * pc.scale + pc.offset, 0.0, 1.0);
                v_uv = uv;
                v_color = color;
            }
        "#,
    }
}

mod text_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r#"
            #version 450
            layout(location = 0) in vec2 v_uv;
            layout(location = 1) in vec4 v_color;
            layout(set = 0, binding = 0) uniform sampler2D tex;
            layout(location = 0) out vec4 f_color;
            void main() {
                f_color = texture(tex, v_uv) * v_color;
            }
        "#,
    }
}
