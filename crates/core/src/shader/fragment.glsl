precision mediump float;

uniform vec3 u_light_pos;
uniform vec3 u_light_color;
uniform vec3 u_view_pos;
uniform float u_light_intensity;

in vec3 v_normal;
in vec3 v_frag_pos;
in vec4 v_color;

out vec4 FragColor;

void main() {
    // Normalize once
    vec3 N = normalize(v_normal);
    vec3 L = normalize(u_light_pos - v_frag_pos);
    vec3 V = normalize(u_view_pos - v_frag_pos);

    // === Ambient ===
    vec3 ambient = 0.7 * u_light_color;

    // === Diffuse ===
    float diff = max(dot(N, L), 0.0);
    vec3 diffuse = 0.5 * diff * u_light_color;

    // === Specular (Blinn-Phong) ===
    vec3 H = normalize(L + V);
    float spec = pow(max(dot(N, H), 0.0), 64.0);     // shininess = 64
    vec3 specular = 0.3 * spec * u_light_color;

    // === Final Color ===
    vec3 lighting = (ambient + diffuse) * v_color.rgb + specular;

    FragColor = vec4(lighting * u_light_intensity, v_color.a);
}
