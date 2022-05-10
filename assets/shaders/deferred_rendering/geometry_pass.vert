#version 330 core
#define NORMAL_MAPPING

layout (location = 0) in vec3   v_position;
layout (location = 1) in vec3   v_normal;

#ifdef NORMAL_MAPPING
layout (location = 2) in vec3   v_tangent_x;
layout (location = 3) in vec3   v_tangent_y;
#endif

layout (location = 4) in vec2   v_texture_coordinates;
layout (location = 5) in int    v_material_id;
layout (location = 6) in int    v_random;

out vec3 f_world_space_position;
out vec3 f_light_space_position;
out vec3 f_normal;
#ifdef NORMAL_MAPPING
out vec3 f_tangent_x;
out vec3 f_tangent_y;
#endif
out vec2 f_texture_coordinates;
flat out int f_material_id;
flat out int f_random;


uniform mat4 u_projview, u_model;
uniform mat4 u_light_projview;

vec3 normalize_vec4(vec4 v) {
    return v.xyz / v.w;
}

int rev_bits(int a) {
    int res = 0;
    for(int i = 0; i < 32; i++) {
        int bit = (a & (1 << i)) >> i;
        res = res | bit << (31 - i);
    }
    return res;
}

void main() {
    // Позиции
    vec4 world_pos = u_model * vec4(v_position, 1.0);
    gl_Position = u_projview * world_pos;

    f_world_space_position = normalize_vec4(world_pos);
    f_light_space_position = normalize_vec4(u_light_projview * world_pos);

    // Базис
    f_normal    = (u_model * vec4(v_normal,      0.0)).xyz;
#ifdef NORMAL_MAPPING
    f_tangent_x = (u_model * vec4(v_tangent_x,   0.0)).xyz;
    f_tangent_y = (u_model * vec4(v_tangent_y,   0.0)).xyz;
#endif

    // Остальное
    f_texture_coordinates = v_texture_coordinates;
    f_material_id = v_material_id;
    f_random = v_random;
}