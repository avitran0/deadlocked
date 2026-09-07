#version 330 core

layout (location = 0) in vec3 a_position;
layout (location = 1) in uvec4 a_joints;
layout (location = 2) in vec4 a_weights;

uniform mat4 u_view;
uniform mat4 u_model;
uniform mat4 u_bones[96];
uniform float u_bone_visibility[96];

out float v_visibility;

void main() {
    mat4 skin =
        a_weights.x * u_bones[a_joints.x] +
        a_weights.y * u_bones[a_joints.y] +
        a_weights.z * u_bones[a_joints.z] +
        a_weights.w * u_bones[a_joints.w];

    v_visibility =
        a_weights.x * u_bone_visibility[a_joints.x] +
        a_weights.y * u_bone_visibility[a_joints.y] +
        a_weights.z * u_bone_visibility[a_joints.z] +
        a_weights.w * u_bone_visibility[a_joints.w];

    gl_Position = u_view * u_model * skin * vec4(a_position, 1.0);
}
