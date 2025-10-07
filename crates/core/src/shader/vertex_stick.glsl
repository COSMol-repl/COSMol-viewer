precision mediump float;

uniform mat4 u_mvp;
uniform mat4 u_model;
uniform mat3 u_normal_matrix;

// per-vertex attributes (单位圆柱模板)
in vec3 a_position;  // 单位圆柱顶点 [-0.5,0.5] x [-radius,radius] x [-0.5,0.5]
in vec3 a_normal;

// per-instance attributes
in vec3 i_start;     // 棒起点
in vec3 i_end;       // 棒终点
in float i_radius;   // 棒半径
in vec4 i_color;     // 棒颜色

// 输出给 fragment shader
out vec3 v_normal;
out vec3 v_frag_pos;
out vec4 v_color;

void main() {
    // 1️⃣ 计算棒方向和长度
    vec3 dir = i_end - i_start;
    float len = length(dir);
    vec3 z_axis = normalize(dir);

    // 2️⃣ 构建局部坐标系
    vec3 tmp = vec3(0.0, 1.0, 0.0);
    if (abs(dot(z_axis, tmp)) > 0.99) tmp = vec3(1.0, 0.0, 0.0); // 避免共线
    vec3 x_axis = normalize(cross(tmp, z_axis));
    vec3 y_axis = cross(z_axis, x_axis);

    mat3 rot = mat3(x_axis, y_axis, z_axis); // 列向量 = x, y, z

    // 3️⃣ 变换顶点到世界空间
    vec3 local_pos = vec3(a_position.x * i_radius, a_position.y * i_radius, a_position.z * len);
    vec4 a_position_transformed = vec4(rot * local_pos + i_start, 1.0);
    vec4 world_pos = u_model * a_position_transformed;
    v_frag_pos = world_pos.xyz;
    vec3 normal_world = rot * a_normal;
    v_normal = normalize(u_normal_matrix * normal_world);
    v_color = i_color;
    gl_Position = u_mvp * a_position_transformed;
}
