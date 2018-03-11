#version 330 core

out vec4 color;

uniform sampler2D renderedTexture;

in vec2 tex_coord_out;

void main(){
    color = texture( renderedTexture, tex_coord_out).rgba;
}
