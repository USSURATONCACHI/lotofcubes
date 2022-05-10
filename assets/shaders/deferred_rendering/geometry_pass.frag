#version 330 core
#define NORMAL_MAPPING

layout (location = 0) out vec3 g_position;
layout (location = 1) out vec3 g_normal;
layout (location = 2) out vec3 g_color;
layout (location = 3) out vec3 g_light;


in vec3     f_world_space_position;
in vec3     f_light_space_position;
in vec3     f_normal;
#ifdef NORMAL_MAPPING
in vec3     f_tangent_x;
in vec3     f_tangent_y;
#endif
in vec2     f_texture_coordinates;
flat in int f_material_id;
flat in int f_random;

struct Material {
    int color_textures_count;
    int normal_textures_count;
    int light_textures_count;

    int color_texture_id;
    int normal_texture_id;
    int light_texture_id;
};

uniform Material    u_materials[50];
uniform sampler2D   u_texture_atlas;
uniform vec2        u_atlas_size;
uniform vec2        u_texture_size;
uniform vec3        u_light_direction;
uniform vec3        u_camera_pos;


const float PI = 3.14159265359;

#include utils.frag

#[del]
vec2 atlas_coords(vec2 texture_coords, int texture_id);
int mod_positive(int a, int b);
vec3 unit_vec(vec3 vec);
#

vec4 get_color() {
    int local_texture_id = mod_positive(f_random, u_materials[f_material_id].color_textures_count);
    int texture_id = u_materials[f_material_id].color_texture_id + local_texture_id;
    return texture2D(u_texture_atlas, atlas_coords(f_texture_coordinates, texture_id));
}
vec3 get_normal_texel() {
    int local_texture_id = mod_positive(f_random, u_materials[f_material_id].normal_textures_count);
    int texture_id = u_materials[f_material_id].normal_texture_id + local_texture_id;
    vec3 normal = texture2D(u_texture_atlas, atlas_coords(f_texture_coordinates, texture_id)).xyz;

    return unit_vec(normal * 2.0 - 1.0);
}
vec3 get_light_texel() {
    int local_texture_id = mod_positive(f_random, u_materials[f_material_id].light_textures_count);
    int texture_id = u_materials[f_material_id].light_texture_id + local_texture_id;

    return texture2D(u_texture_atlas, atlas_coords(f_texture_coordinates, texture_id)).xyz;
}

void main() {
    g_position = f_world_space_position;

#ifdef NORMAL_MAPPING
    vec3 normal_texel = get_normal_texel();
    g_normal = normal_texel.x * f_tangent_x + normal_texel.y * f_tangent_y + normal_texel.z * f_normal;
#else
    g_normal = f_normal;
#endif

    vec4 color = get_color();
    g_color = color.rgb * color.w;

    g_light = get_light_texel();
}