#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_color;
uniform mat4 u_view;
out vec3 v_pos;
out vec3 v_color;

void main() {
    gl_Position = u_view * vec4(a_pos, 1.0);
    v_pos = a_pos;
    v_color = a_color;
}
