precision mediump float;

uniform mat4 u_mvp;
uniform mat4 u_model;
uniform mat3 u_normal_matrix;

in vec3 a_position;
in vec3 a_normal;
in vec4 a_color;

out vec3 v_normal;
out vec3 v_frag_pos;
out vec4 v_color;

void main() {
    vec4 world_pos = u_model * vec4(a_position, 1.0);
    v_frag_pos = world_pos.xyz;
    v_normal = normalize(u_normal_matrix * a_normal);
    v_color = a_color;
    gl_Position = u_mvp * vec4(a_position, 1.0);
}