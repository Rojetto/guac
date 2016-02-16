#version 330

in vec3 position;
in vec3 normal;
in vec4 color;

out vec3 frag_normal;
out vec4 frag_color;

uniform mat4 model;
uniform mat4 view;
uniform mat4 perspective;

void main() {
    gl_Position = perspective * view * model * vec4(position, 1.0);
    frag_normal = mat3(transpose(inverse(model))) * normal;
    frag_color = color;
}
