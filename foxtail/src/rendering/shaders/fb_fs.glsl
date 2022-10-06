#version 450

out vec4 color;

in vec2 f_uv;

uniform sampler2D tex;

void main() {
    color = texture(tex, f_uv);
}
