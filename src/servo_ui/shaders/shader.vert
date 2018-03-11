#version 330 core

in vec3 in_position;
in vec2 tex_coord;
out vec2 tex_coord_out;

void main() {
    gl_Position = vec4(in_position, 1);
    tex_coord_out = tex_coord;
}
