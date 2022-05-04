layout (location = 0) in vec3  v_model_space_position;    // Vertex position in model space
layout (location = 1) in vec3  v_normal;
#ifdef NORMAL_MAPPING
layout (location = 2) in vec3  v_tangent_x;
#endif
//layout (location = 3) in vec3  v_tangent_y;
layout (location = 4) in vec2  v_texture_coordinates;  // Position of the vertex on texture (0.0 - 1.0)
layout (location = 5) in float v_material_id;          // Normalized material id (actually its 1 / (id + 1))

out vec3  f_world_space_position;
out vec3  f_normal;
#ifdef NORMAL_MAPPING
out vec3  f_tangent_x;
#endif
//out vec3  f_tangent_y;
out vec2  f_texture_coordinates;
out float f_material_id;
out vec3  f_light_space_pos;  // Position of vertex in light space (Light sourse's POV)

uniform mat4 u_projection, u_view, u_model;
uniform mat4 u_light_projview;    // Projection * View matrix of light source

//Functions

// Hyperbolically curved variable. "a" is in bounds (-inf;+inf), curve(a) is in bounds (-1;+1)
// mul - multiplier - scale of equation in xy
float curve(float a, float mul) {
    float abs = abs(a);
    float sign = a >= 0 ? 1.0 : -1.0;
    return mul * (1.0 - 1 / (abs / mul + 1.0)) * sign;
}

vec3 normalize_vec4(vec4 vector) {  return vector.xyz / vector.w;  }

vec3 unit_vec(vec3 vector) {  return vector / length(vector);  }


//Main

void main() {
    // Position of vertex in world coordinates
    vec4 vertex_world_pos = u_model * vec4(v_model_space_position, 1.0);

    // Transforming it into unit cube coordinates
    gl_Position = u_projection * u_view * vertex_world_pos;

    // Position of vertex in light source's POV (Euclidian)
    f_light_space_pos = normalize_vec4(u_light_projview * vertex_world_pos);
    // We need to curve it to apply hyperbolic curivature to fit all the numbers to finite sized segment (-1;+1)
    //f_light_space_pos = vec3(curve(f_light_space_pos.x, 1.0), curve(f_light_space_pos.y, 1.0), curve(f_light_space_pos.z, 1.0));

    // Transforming basis (normal + two tangent vectors) into world coordinates
    f_normal    = unit_vec(  (u_model * vec4(v_normal, 0.0)).xyz    );
    #ifdef NORMAL_MAPPING
    f_tangent_x = unit_vec(  (u_model * vec4(v_tangent_x, 0.0)).xyz );
    #endif
    //f_tangent_y = unit_vec(  (u_model * vec4(v_tangent_y, 0.0)).xyz );

    // Passing world position of vertex to fragment shader
    f_world_space_position = normalize_vec4(vertex_world_pos);

    // These are just passing through
    f_texture_coordinates = v_texture_coordinates;
    f_material_id = v_material_id;
}