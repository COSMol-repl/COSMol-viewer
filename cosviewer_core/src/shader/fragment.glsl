precision mediump float;

uniform vec3 u_light_pos;
uniform vec3 u_light_color;
uniform vec3 u_view_pos;
uniform float u_light_intensity;

in vec3 v_normal;
in vec4 v_color;
in vec3 v_frag_pos;

out vec4 FragColor;
void main() {
    vec3 normal = normalize(v_normal);                         // 归一化法线
    vec3 light_dir = normalize(u_light_pos - v_frag_pos);      // 计算从片元到光源的方向
    vec3 view_dir = normalize(u_view_pos - v_frag_pos);        // 计算从片元到相机的方向
    vec3 halfway_dir = normalize(light_dir + view_dir);        // halfway 向量用于 Blinn-Phong 高光

    float diff = max(dot(normal, light_dir), 0.0);             // 漫反射项（Lambert）
    float spec = pow(max(dot(normal, halfway_dir), 0.0), 32.0); // 高光项，32 是 shininess 参数

    vec3 ambient = 0.3 * u_light_color;                         // 环境光，固定系数0.2
    vec3 diffuse = diff * u_light_color;                        // 漫反射
    // vec3 specular = spec * u_light_color;                       // 高光

    vec3 light = (ambient + diffuse) * u_light_intensity; // 叠加光照并乘以强度
    // vec3 light = (ambient + diffuse + specular) * u_light_intensity; // 叠加光照并乘以强度

    FragColor = vec4(v_color.rgb * light, v_color.a);
}