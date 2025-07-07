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
    vec3 normal = normalize(v_normal);
    vec3 light_dir = normalize(u_light_pos - v_frag_pos);
    vec3 view_dir = normalize(u_view_pos - v_frag_pos);

    // === 环境光 ===
    vec3 ambient = 0.7 * u_light_color;

    // === 漫反射 ===
    float diff = max(dot(normal, light_dir), 0.0);
    vec3 diffuse = 0.5 * diff * u_light_color;

    // === 高光 ===
    vec3 halfway_dir = normalize(light_dir + view_dir); // Blinn 模型
    float spec = pow(max(dot(normal, halfway_dir), 0.0), 64.0); // shininess 可调
    vec3 specular = 0.3 * spec * u_light_color; // 强度可调

    // === 最终颜色 ===
    vec3 lighting = (ambient + diffuse) * v_color.rgb + specular;
    FragColor = vec4(lighting * u_light_intensity, v_color.a);
}
