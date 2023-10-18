#version 450

out vec4 color;

in vec2 f_uv;

layout(binding = 0) uniform sampler2D tex;

void main() {
    color = texture(tex, f_uv);
}
