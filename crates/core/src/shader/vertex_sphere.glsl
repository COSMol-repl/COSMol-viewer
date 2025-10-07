precision mediump float;

uniform mat4 u_mvp;
uniform mat4 u_model;
uniform mat3 u_normal_matrix;

in vec3 a_position;
in vec3 a_normal;
in vec3 i_position;  // instance: sphere center
in float i_radius;   // instance: sphere radius
in vec4 i_color;     // instance: sphere color

out vec3 v_normal;
out vec3 v_frag_pos;
out vec4 v_color;

void main() {
    vec4 a_position_transformed = vec4(a_position * i_radius + i_position, 1.0);
    vec4 world_pos = u_model * a_position_transformed;
    v_frag_pos = world_pos.xyz;
    v_normal = normalize(u_normal_matrix * a_normal);
    v_color = i_color;
    gl_Position = u_mvp * a_position_transformed;
}