#version 330 core
out vec4 out_color;

in vec2 f_texture_coords;

uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform sampler2D g_color;
uniform sampler2D g_light;

uniform vec3 u_light_direction;
uniform vec3 u_camera_pos;

vec3 unit_vec(vec3 v) {
    return v / length(v);
}

float pow16_easy(float x) {
    x *= x; //2
    x *= x; //4
    x *= x; //8
    x *= x; //16
    return x;
}

void main() {
    vec3 f_world_space_position = texture2D(g_position, f_texture_coords).xyz;
    vec3 f_normal       = texture2D(g_normal, f_texture_coords).xyz;
    vec3 f_color        = texture2D(g_color, f_texture_coords).rgb;
    vec3 f_light        = texture2D(g_light, f_texture_coords).xyz;

    //Если нормаль нулевая, значит здесь нет фрагмента
    if( length(f_normal) < 0.00001 ) {
        //Тут нужно просто нарисовать небо, позже этим займусь нормально
        out_color = vec4(0.55, 0.87, 0.95, 1.0);
        return;
    }

    float diffuse_light = max(0.0, -dot(f_normal,  u_light_direction));

    vec3 local_light_ray = unit_vec(u_camera_pos - f_world_space_position);
    vec3 reflected_light_ray = local_light_ray - 2.0 * f_normal * dot(local_light_ray, f_normal);

    float specular = max(  dot(reflected_light_ray, u_light_direction), 0.0 );
    specular = pow16_easy(specular);


    out_color = vec4(f_color, 1.0) * (diffuse_light * 0.9 + 0.3) + 0.5 * specular;
}