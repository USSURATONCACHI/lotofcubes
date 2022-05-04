// Functions
vec3 unit_vec(vec3 vector) {  return vector / length(vector);  }

//Mod with only positive part. positive_mod(7, 5) is 2; positive_mod(-1, 5) is 4
int positive_mod(int x, int max) { return ((x % max) + max) % max; }
float pow16_easy(float x) {
    x *= x; //2
    x *= x; //4
    x *= x; //8
    x *= x; //16
    return x;
}

float curve(float a, float mul) {
    float abs = abs(a);
    float sign = a >= 0 ? 1.0 : -1.0;
    return mul * (1.0 - 1 / (abs / mul + 1.0)) * sign;
}


float map(float value, float min1, float max1, float min2, float max2) {
    return min2 + (value - min1) * (max2 - min2) / (max1 - min1);
}
vec3 map(vec3 value, vec3 min1, vec3 max1, vec3 min2, vec3 max2) {
    return min2 + (value - min1) * (max2 - min2) / (max1 - min1);
}


int material_color_textures_count   (int mat_id) { return u_textures_data[mat_id * 6 + 0]; }
int material_normal_textures_count  (int mat_id) { return u_textures_data[mat_id * 6 + 1]; }
int material_lightmap_textures_count(int mat_id) { return u_textures_data[mat_id * 6 + 2]; }
int material_color_texture_id       (int mat_id) { return u_textures_data[mat_id * 6 + 3]; }
int material_normal_texture_id      (int mat_id) { return u_textures_data[mat_id * 6 + 4]; }
int material_lightmap_texture_id    (int mat_id) { return u_textures_data[mat_id * 6 + 5]; }

//N is number of texture (loops over),  mat_id is id of material
//type - type of texture: 0 is color texture, 1 is normal texture, 2 is lightmap texture. Other is undefined behavior
int nth_material_texture (int n, int mat_id, int type) {
    return  u_textures_data[mat_id * 6 + 3 + type] +
    positive_mod(  n,  u_textures_data[mat_id * 6 + 0 + type]  );
}

//Get the position of texture on atlas
vec2 get_texture_pos(int tex_id) {
    int count = int(floor(u_atlas_size.x / u_texture_size.x + 0.01)); //Count of full textures fit in one atlas width
    return vec2(  float(tex_id % count), float(tex_id / count)  ) * u_texture_size / u_atlas_size;
}

//Temporary
int random() {  //Noise based on block position
    int ix = int(floor(f_world_space_position.x + 0.5));
    int iy = int(floor(f_world_space_position.y + 0.5));
    int iz = int(floor(f_world_space_position.z + 0.5));

    int rand_seed = ix*iy*iz + ix*iy + ix*ix + iy*iz + ix + iy + iz;
    rand_seed = (rand_seed ^ 0xF7B2132A) * 0xBB12A45F;
    rand_seed = rand_seed ^ int(sqrt(float(rand_seed))) ^ (rand_seed * rand_seed);
    rand_seed = rand_seed ^ int(sqrt(float(rand_seed))) ^ (rand_seed * rand_seed);
    return 0;
}


//Position of texel. tex_coords - position of TEXEL ON TEXTURE. tex_pos - position of TEXTURE ON ATLAS
vec2 get_texel_pos(vec2 tex_coords, vec2 tex_pos) {
    //Clamp is needed to make sure texel won't get out of current texture bounds
    return clamp(tex_coords, vec2(0.001), vec2(0.999)) * u_texture_size / u_atlas_size + tex_pos;
}
vec4 get_texel(vec2 tex_coords, int tex_id) {
    return texture2D(u_texture_atlas, get_texel_pos(tex_coords, get_texture_pos(tex_id) ));
}