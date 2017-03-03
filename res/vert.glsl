#version 140

in vec3 pos;
in vec3 color;

out vec3 f_color;

void main() {
    gl_Position = vec4(pos, 1.0);
    f_color = color;
}