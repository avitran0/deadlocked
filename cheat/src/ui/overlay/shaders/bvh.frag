#version 330 core
in vec3 v_pos;
in vec3 v_color;
out vec4 color;
uniform vec3 u_view_pos;
uniform float u_depth_threshold;

void main() {
    float distance_to_view = length(u_view_pos - v_pos);
    if (distance_to_view >= u_depth_threshold) {
        discard;
    }

    float fade_width = max(u_depth_threshold * 0.25, 1.0);
    float fade_start = max(u_depth_threshold - fade_width, 0.0);
    float alpha = 1.0 - smoothstep(fade_start, u_depth_threshold, distance_to_view);
    color = vec4(v_color, alpha);
}
