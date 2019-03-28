pub mod lines_vsm {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
    vec2 norm_coords = (position + vec2(0.5)) / vec2(1024);
    gl_Position = vec4(norm_coords, 0.0, 1.0);
    // gl_Position = vec4((position-512.0) / 512, 0.0, 1.0);
}
"
    }
}

pub mod lines_fsm {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}

"
    }
}
