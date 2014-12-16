#version 130

in vec2 tex_coord;
out vec4 color;
uniform sampler2D texture;

void main() {
    color = texture2D(texture, tex_coord);
}
