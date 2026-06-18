#version 330 core
layout (location = 0) in vec3 aPos;

layout(std140) uniform Camera {
    mat4 view;
    mat4 projection;
};

layout(std140) uniform Model {
    mat4 modelMatrix;
};

void main()
{
    vec4 position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    vec4 out_pos =  projection * view * modelMatrix * position;

    gl_Position = out_pos;
}
