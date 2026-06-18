#version 330 core

out vec4 FragColor;

layout(std140) uniform Material {
    vec4 color;
};

void main()
{
    FragColor = color;
} 
