#version 330 core

layout (location = 0) in vec3 pos;

uniform mat4 projection, view, model;

float curve(float a, float mul) {
    float abs = abs(a);
    float sign = a >= 0 ? 1.0 : -1.0;
    return mul * (1.0 - 1 / (abs / mul + 1.0)) * sign;
}

void main()
{
    gl_Position = projection * view * model * vec4(pos, 1.0);
    //gl_Position = vec4(curve(gl_Position.x, 1.0), curve(gl_Position.y, 1.0), curve(gl_Position.z, 1.0), 1.0);
}