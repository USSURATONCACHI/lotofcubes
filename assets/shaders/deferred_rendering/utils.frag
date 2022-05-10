vec2 atlas_coords(vec2 texture_coords, int texture_id) {
    //Количество текстур, помещающихся в атлас (по ширине и по высоте)
    ivec2 count = ivec2(floor(u_atlas_size / u_texture_size + 0.01));
    vec2 texture_pos = vec2(  float(texture_id % count.x), float(texture_id / count.y)  );
    return (texture_pos + texture_coords) * u_texture_size / u_atlas_size;
}

int mod_positive(int a, int b) {
    return ((a % b) + b) % b;
}

vec3 unit_vec(vec3 vec) {
    return vec / length(vec);
}