precision mediump float;

uniform mat4 u_mvp;
uniform mat4 u_model;
uniform mat4 u_normal_matrix;

in vec3 a_position;
in vec3 a_normal;
in vec3 i_position;  // instance: sphere center
in float i_radius;   // instance: sphere radius
in vec4 i_color;     // instance: sphere color

out vec3 v_normal;
out vec4 v_color;

void main() {
    vec3 world_pos = a_position * i_radius + i_position;
    gl_Position = u_mvp * vec4(world_pos, 1.0);

    v_normal = mat3(u_normal_matrix) * a_normal;
    v_color = i_color;
}
