#version 330 core

uniform vec4 u_visible_color;
uniform vec4 u_invisible_color;
in float v_visibility;
out vec4 frag_color;

void main() {
    frag_color = mix(u_invisible_color, u_visible_color, clamp(v_visibility, 0.0, 1.0));
}
