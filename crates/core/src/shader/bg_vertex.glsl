precision mediump float;

uniform vec3 background_color;

const vec2 verts[6] = vec2[6](
    vec2(-1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0)
);
out vec4 v_color;
void main() {
    v_color = vec4(background_color, 1.0);
    gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
}