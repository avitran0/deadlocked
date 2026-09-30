#version 330 core
in vec3 v_pos;
in vec3 v_color;
out vec4 color;
uniform vec3 u_view_pos;
uniform float u_depth_threshold;

void main() {
    if (length(u_view_pos - v_pos) > u_depth_threshold) {
        discard;
    }

    color = vec4(v_color, 1.0);
}
