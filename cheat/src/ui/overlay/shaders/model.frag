#version 330 core

uniform vec4 u_visible_color;
uniform vec4 u_invisible_color;
uniform bool u_visible_only;
in float v_visibility;
out vec4 frag_color;

void main() {
    if (u_visible_only && v_visibility < 0.5) {
        discard;
    }
    frag_color = mix(u_invisible_color, u_visible_color, clamp(v_visibility, 0.0, 1.0));
}
