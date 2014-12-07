#version 130

in vec3 position;
in vec2 texture_uv;
out vec2 tex_coord;
uniform mat4 projection_view;
uniform mat4 model;

void main(void) {
    vec4 v = vec4(position, 1.0);
    vec4 p = projection_view * model * v;
    tex_coord = texture_uv;
    gl_Position = p;
}
