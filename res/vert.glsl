#version 140

in vec3 pos;
in vec3 color;

out vec3 f_color;

uniform mat4 P;
uniform mat4 M;

void main() {
    gl_Position = P * M * vec4(pos, 1.0);
    f_color = color;
}
