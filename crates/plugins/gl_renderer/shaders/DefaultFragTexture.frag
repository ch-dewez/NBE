#version 330 core

in vec3 ourColor;
in vec2 ourTexCoord;
out vec4 FragColor;

uniform sampler2D ourTexture;

layout(std140) uniform Material {
    vec4 color;
};

void main()
{
    FragColor = texture(ourTexture, ourTexCoord) * vec4(ourColor, 1.0);
} 
