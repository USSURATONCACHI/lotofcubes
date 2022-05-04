out vec4 out_color;

// f is for fragment
in vec3  f_world_space_position;      // Fragment position in world coordinates
in vec3  f_normal;
#ifdef NORMAL_MAPPING
in vec3  f_tangent_x;
#endif
//in vec3  f_tangent_y;
in vec2  f_texture_coordinates;
in float f_material_id;               // Normalized (1 / (id + 1)) id of material
in vec3  f_light_space_pos;           // Position of fragment in light source coordinates (light source's POV)

// u is for uniform
uniform sampler2D u_texture_atlas;    // Atlas with all the textures in it
uniform sampler2D u_shadow_map;       // Depth texture of light source's POV

/* All the data about in-game textures' positions in the atlas.
 Every texture contains six variables:
 0 - color textures count      (just a regular textures)
 1 - normal textures count
 2 - light map textures count  (these ones contain information about texture's interaction with light, like reflection or brigtness)
 3 - first color texture id    - ID in the atlas. Next <color textures count> are placed in order
 4 - first normal texture id
 5 - first light map texture id

 So to get j-th parameter from i-th texture you need to do textures_data[i * 6 + j];
 (For example, first color texture ID of 5th in-game texture is textures_data[5 * 6 + 3])   */
uniform int  u_textures_data[100];

uniform vec2 u_atlas_size;       // Image width and height
uniform vec2 u_texture_size;     // Single texture width and height

uniform vec3 u_light_direction;   // Vector pointing light direction (in world coorinates)
uniform vec3 u_camera_pos;        // Position of camera (in world coordinates)

const float PI = 3.14159265359;

#include<main_utils.glsl>

// Returns coefficient (0 - 1) direct light force
float calculate_light() {
    float MAX_BIAS = 0.000025 ;
    float MIN_BIAS = 0.0000025;

    // Normalized and mapped
    vec3 point = map( f_light_space_pos, vec3(-1.0), vec3(1.0), vec3(0.0), vec3(1.0) );

    float nl_dot = dot(u_light_direction, f_normal);      // Normal-light dot product
    if (nl_dot >= 0.0)      // If dot is not negative, it means surface is turned away from the light or is perpendicular
    return 0.0;   // So no light fall on this surface

    float point_depth = point.z;  //Depth of this point

    if(point_depth > 1.0) return 1.0;

    //float bias = max(MAX_BIAS * (1.0 - nl_dot), MIN_BIAS);   //This one looks worse
    //float bias = 0.000125; //Depth offset
    float bias_multiplier = length(f_light_space_pos.xy) + 1.0;
    float bias = max( 0.000125*tan(acos(-nl_dot)), MIN_BIAS ) * bias_multiplier / (point_depth * point_depth);

    vec2  pixel_size = vec2(1.0) / textureSize(u_shadow_map, 0);
    float smoothed = 0.0;
    for(int dx = -1; dx <= 1; dx++) {
        for(int dy = -1; dy <= 1; dy++ ) {
            //Depth from the shadow map
            vec2 offset = vec2(dx, dy);
            vec2 point = point.xy + (offset / max(length(offset), 1.0)) * pixel_size;
            float closest_depth = texture2D(u_shadow_map, point).r;

            smoothed += point_depth - bias >= closest_depth ? 0.0 : 1.0; //0.0 - no light, 1.0 - full light
        }
    }
    smoothed /= 9.0;    //Getting average of all nine points

    // -nl_dot - is amount scattered light in this point
    // smoothed - amount of direct light in this point
    return smoothed;
}

// Main

void main() {
    int rand = random();   // TMP

    int material_id = int(round(1.0 / f_material_id - 1.0));    // ID of current material

    // Color, normal (light is not taken in count)
    vec4 fragment_color        = get_texel(  f_texture_coordinates, nth_material_texture(rand, material_id, 0)  );

    #ifdef NORMAL_MAPPING
    vec4 fragment_normal_texel = get_texel(  f_texture_coordinates, nth_material_texture(rand, material_id, 1)  );
    // Transforming color data to normal vector coordinates
    vec3 local_normal = fragment_normal_texel.xyz * 2.0 - 1.0;
    vec3 mapped_normal = unit_vec(
        local_normal.z * f_normal +
        local_normal.y * cross(f_normal, f_tangent_x) +
        local_normal.x * f_tangent_x
    );  // Mapped normal in the world space
    #else
    vec3 mapped_normal = f_normal;
    #endif


    // Calculating light - shadow and reflections
    float camera_distance = length(u_camera_pos - f_world_space_position);      // Distance from camera to fragment
    //float camera_distance_log = log(camera_distance)/log(2.0);
    vec3 local_light_ray = unit_vec(u_camera_pos - f_world_space_position);     // Ray from camera to fragment (world space)
    vec3 reflected_light_ray = local_light_ray - 2.0 * mapped_normal * dot(local_light_ray, mapped_normal); //  Reflected ray


    float direct_light_coef    = calculate_light(); // Coefficient (0 - 1) of direct light force
    float scattered_light_coef = max(  -dot(mapped_normal, u_light_direction), 0.0  );      // Likewise
    float reflected_light_coef = max(  dot(reflected_light_ray, u_light_direction), 0.0 );  // Likewise
    reflected_light_coef *= direct_light_coef; //There is no reflection if fragment is in shadow
    reflected_light_coef = pow16_easy(reflected_light_coef);

    //out_color = vec4( mapped_normal, 1.0 );

    out_color = vec4(
        fragment_color.rgb * (
            scattered_light_coef * direct_light_coef * 0.8 +        //Amount of direct light
            scattered_light_coef * 0.32 +                           //Amount of ambient light
            0.23) +                                                 //Amount of it's own light
        0.5 * reflected_light_coef,

        fragment_color.a
    );
    //out_color = vec4(mapped_normal, 1.0);
    //out_color = vec4(vec3(texture2D(u_shadow_map, f_texture_coordinates)), 1.0);
}