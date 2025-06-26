use std::sync::Arc;

use eframe::{egui::{self, mutex::Mutex}, egui_glow, glow};

pub struct Triangle {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    rotating_triangle: Arc<Mutex<RotatingTriangle>>,
    angle: f32,
}

impl Triangle {
    pub fn new<'a>(gl: Arc<eframe::glow::Context>) -> Option<Self> {
        Some(Self {
            rotating_triangle: Arc::new(Mutex::new(RotatingTriangle::new(&gl)?)),
            angle: 0.0,
        })
    }
}

impl Triangle {
    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2 {
                x: ui.available_width(),
                y: ui.available_height(),
            },
            egui::Sense::drag(),
        );

        self.angle += response.drag_motion().x * 0.01;

        // Clone locals so we can move them into the paint callback:
        let angle = self.angle;
        let rotating_triangle = self.rotating_triangle.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            rotating_triangle.lock().paint(painter.gl(), angle);
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }
}

struct RotatingTriangle {
    program: glow::Program,
    program_bg: glow::Program,
    vertex_array: glow::VertexArray,
}

#[expect(unsafe_code)] // we need unsafe code to use glow
impl RotatingTriangle {
    fn new(gl: &glow::Context) -> Option<Self> {
        use glow::HasContext as _;

        let shader_version = egui_glow::ShaderVersion::get(gl);

        unsafe {
            let program_bg = gl.create_program().expect("Cannot create program");
            let program = gl.create_program().expect("Cannot create program");

            if !shader_version.is_new_shader_interface() {
                // log::warn!(
                //     "Custom 3D painting hasn't been ported to {:?}",
                //     shader_version
                // );
                // return None;
            }

            let (vertex_shader_source, fragment_shader_source) = (
                r#"
                    const vec2 verts[3] = vec2[3](
                        vec2(0.0, 1.0),
                        vec2(-1.0, -1.0),
                        vec2(1.0, -1.0)
                    );
                    const vec4 colors[3] = vec4[3](
                        vec4(1.0, 0.0, 0.0, 1.0),
                        vec4(0.0, 1.0, 0.0, 1.0),
                        vec4(0.0, 0.0, 1.0, 1.0)
                    );
                    out vec4 v_color;
                    uniform float u_angle;
                    void main() {
                        v_color = colors[gl_VertexID];
                        gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
                        gl_Position.x *= cos(u_angle);
                    }
                "#,
                r#"
                    precision mediump float;
                    in vec4 v_color;
                    out vec4 out_color;
                    void main() {
                        out_color = v_color;
                    }
                "#,
            );

            let (vertex_shader_bg, fragment_shader_bg) = (
                r#"
                    const vec2 verts[6] = vec2[6](
                        vec2(-1.0, 1.0),
                        vec2(1.0, 1.0),
                        vec2(-1.0, -1.0),
                        vec2(1.0, 1.0),
                        vec2(-1.0, -1.0),
                        vec2(1.0, -1.0)
                    );
                    const vec4 color = vec4(0.19, 0.19, 0.19, 1.0);
                    out vec4 v_color;
                    void main() {
                        v_color = color;
                        gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
                    }
                "#,
                r#"
                    precision mediump float;
                    in vec4 v_color;
                    out vec4 out_color;
                    void main() {
                        out_color = v_color;
                    }
                "#,
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shader_bg =     [
                (glow::VERTEX_SHADER, vertex_shader_bg),
                (glow::FRAGMENT_SHADER, fragment_shader_bg),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(
                        shader,
                        &format!(
                            "{}\n{}",
                            shader_version.version_declaration(),
                            shader_source
                        ),
                    );
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile custom_3d_glow {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );

                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let shaders_bg: Vec<_> = shader_bg
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(
                        shader,
                        &format!(
                            "{}\n{}",
                            shader_version.version_declaration(),
                            shader_source
                        ),
                    );
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile custom_3d_glow {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );

                    gl.attach_shader(program_bg, shader);
                    shader
                })
                .collect();

            gl.link_program(program_bg);
            assert!(
                gl.get_program_link_status(program_bg),
                "{}",
                gl.get_program_info_log(program_bg)
            );

            for shader in shaders_bg {
                gl.detach_shader(program_bg, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            Some(Self {
                program,
                program_bg,
                vertex_array,
            })
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn paint(&self, gl: &glow::Context, angle: f32) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(Some(self.program_bg));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            gl.use_program(Some(self.program));
            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "u_angle").as_ref(),
                angle,
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}
