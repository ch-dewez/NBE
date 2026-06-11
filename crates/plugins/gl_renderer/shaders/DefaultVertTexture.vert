#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

layout(std140) uniform Camera {
    mat4 view;
    mat4 projection;
};

layout(std140) uniform Model {
    mat4 modelMatrix;
};

out vec3 ourColor;
out vec2 ourTexCoord;

void main()
{
    vec4 position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    vec4 out_pos =  projection * view * modelMatrix * position;
    ourColor = aColor;
    ourTexCoord = aTexCoord;

    gl_Position = out_pos;
}
