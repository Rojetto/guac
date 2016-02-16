#version 330

in vec3 frag_normal;
in vec4 frag_color;

out vec4 color;

void main() {
    float a = (dot(frag_normal, normalize(vec3(-2.0, -1.0, -3.0))) + 1) / 2.0;
    //color = mix(vec4(0.1, 0.1, 0.1, 1.0), vec4(0.7, 0.7, 0.7, 1.0), a);
    color = frag_color;
}
