#version 330 core

out vec4 out_color;

// f is for fragment
in vec2  f_tex_pos;      // Fragment position in world coordinates

uniform sampler2D u_shadow_map;

void main() {
    out_color = vec4(vec3(texture2D(u_shadow_map, f_tex_pos).r), 1.0);
}