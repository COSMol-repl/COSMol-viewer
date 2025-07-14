use glam::Mat3;
use glam::Mat4;
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use eframe::{
    egui::{self, Vec2, mutex::Mutex},
    egui_glow, glow,
};
use glam::{Quat, Vec3};

use crate::Scene;

pub struct Canvas {
    shader: Arc<Mutex<Shader>>,
    camera_state: CameraState,
}

impl Canvas {
    pub fn new<'a>(gl: Arc<eframe::glow::Context>, scene: Scene) -> Option<Self> {
        Some(Self {
            shader: Arc::new(Mutex::new(Shader::new(&gl, scene)?)),
            camera_state: CameraState::new(1.0),
        })
    }

    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2 {
                x: ui.available_width(),
                y: ui.available_height(),
            },
            egui::Sense::drag(),
        );

        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);

        // 正值表示向上滚动，通常是“缩小”，负值是放大
        if scroll_delta != 0.0 {
            self.camera_state.scale *= (1.0 + scroll_delta * 0.001).clamp(0.1, 10.0);
        }

        self.camera_state = rotate_camera(self.camera_state, response.drag_motion());

        // Clone locals so we can move them into the paint callback:
        let cuboid_shader = self.shader.clone();

        let aspect_ratio = rect.width() / rect.height();
        let camera_state = self.camera_state.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            cuboid_shader
                .lock()
                .paint(painter.gl(), aspect_ratio, camera_state);
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }

    pub fn update_scene(&mut self, scene: Scene) {
        self.shader.lock().update_scene(scene);
        // println!("Scene updated in Canvas");
    }
}

struct Shader {
    program: glow::Program,
    program_bg: glow::Program,
    vertex_array: glow::VertexArray,
    vertex3d: Vec<Vertex3d>,
    indices: Vec<u32>,
    background_color: [f32; 3],
    vbo: glow::Buffer,
    element_array_buffer: glow::Buffer,
    dirty: bool,
}

#[expect(unsafe_code)] // we need unsafe code to use glow
impl Shader {
    fn new(gl: &glow::Context, scene: Scene) -> Option<Self> {
        use glow::HasContext as _;

        let shader_version = egui_glow::ShaderVersion::get(gl);

        let background_color = scene.background_color;

        let default_color = [1.0, 1.0, 1.0, 1.0];

        let mut vertex3d: Vec<Vertex3d> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut vertex_offset = 0u32;

        for mesh in scene._get_meshes() {
            vertex3d.extend(mesh.vertices.iter().enumerate().map(|(i, pos)| {
                Vertex3d {
                    position: *pos,
                    normal: mesh.normals[i],
                    color: mesh
                        .colors
                        .as_ref()
                        .and_then(|colors| colors.get(i))
                        .unwrap_or(&default_color)
                        .clone(),
                }
            }));

            indices.extend(mesh.indices.iter().map(|&i| i + vertex_offset));
            vertex_offset += mesh.vertices.len() as u32;
        }

        unsafe {
            let program_bg = gl.create_program().expect("Cannot create program");
            let program = gl.create_program().expect("Cannot create program");

            if !shader_version.is_new_shader_interface() {
                println!(
                    "Custom 3D painting hasn't been ported to {:?}",
                    shader_version
                );
                return None;
            }

            let (vertex_shader_source, fragment_shader_source) = (
                include_str!("./vertex.glsl"),
                include_str!("./fragment.glsl"),
            );

            let (vertex_shader_bg, fragment_shader_bg) = (
                include_str!("./bg_vertex.glsl"),
                include_str!("./bg_fragment.glsl"),
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shader_bg = [
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
                        "Failed to compile custom_3d_glow_bg {shader_type}: {}",
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

            // VAO
            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vertex_array));

            // VBO
            let vertex_buffer = gl.create_buffer().expect("Cannot create vertex buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertex3d),
                glow::DYNAMIC_DRAW,
            );

            // EBO
            let ebo = gl.create_buffer().expect("Cannot create element buffer");
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&indices),
                glow::DYNAMIC_DRAW,
            );

            let stride = std::mem::size_of::<Vertex3d>() as i32;
            let pos_loc = gl.get_attrib_location(program, "a_position").unwrap();
            let normal_loc = gl.get_attrib_location(program, "a_normal").unwrap();
            let color_loc = gl.get_attrib_location(program, "a_color").unwrap();

            gl.enable_vertex_attrib_array(pos_loc);
            gl.vertex_attrib_pointer_f32(pos_loc, 3, glow::FLOAT, false, stride, 0);

            gl.enable_vertex_attrib_array(normal_loc);
            gl.vertex_attrib_pointer_f32(normal_loc, 3, glow::FLOAT, false, stride, 3 * 4);

            gl.enable_vertex_attrib_array(color_loc);
            gl.vertex_attrib_pointer_f32(color_loc, 4, glow::FLOAT, false, stride, 6 * 4);

            gl.bind_vertex_array(None);

            gl.use_program(Some(program));

            Some(Self {
                program,
                program_bg,
                vertex3d,
                vertex_array,
                indices,
                background_color,
                dirty: false,
                vbo: vertex_buffer,
                element_array_buffer: ebo,
            })
        }
    }

    fn update_scene(&mut self, scene_data: Scene) {
        self.background_color = scene_data.background_color;
        self.vertex3d.clear();
        self.indices.clear();

        let mut vertex_offset = 0u32;

        for mesh in scene_data._get_meshes() {
            self.vertex3d
                .extend(mesh.vertices.iter().enumerate().map(|(i, pos)| {
                    Vertex3d {
                        position: *pos,
                        normal: mesh.normals[i],
                        color: mesh
                            .colors
                            .as_ref()
                            .and_then(|colors| colors.get(i))
                            .unwrap_or(&[1.0, 1.0, 1.0, 1.0])
                            .clone(),
                    }
                }));

            self.indices
                .extend(mesh.indices.iter().map(|&i| i + vertex_offset));
            vertex_offset += mesh.vertices.len() as u32;
        }

        self.dirty = true;
    }

    fn paint(&mut self, gl: &glow::Context, aspect_ratio: f32, camera_state: CameraState) {
        use glow::HasContext as _;

        let camera_position = -camera_state.direction * camera_state.distance;
        let camera_direction = camera_state.direction;
        let camera_up = camera_state.up;
        let camera = Camera::new(
            [camera_position.x, camera_position.y, camera_position.z],
            [camera_direction.x, camera_direction.y, camera_direction.z],
            [camera_up.x, camera_up.y, camera_up.z],
            45.0,
            camera_state.scale,
        );

        let light = Light {
            position: [2.0, -3.0, 2.0],
            color: [1.0, 0.9, 0.9],
            intensity: 1.0,
        };

        unsafe {
            // 背面剔除 + 深度测试
            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
            gl.front_face(glow::CCW);

            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);

            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            // === 绘制背景 ===
            gl.disable(glow::DEPTH_TEST); // ✅ 背景不需要深度
            gl.use_program(Some(self.program_bg));
            gl.uniform_3_f32_slice(
                gl.get_uniform_location(self.program_bg, "background_color")
                    .as_ref(),
                &self.background_color,
            );
            gl.draw_arrays(glow::TRIANGLES, 0, 6);

            // === 绘制场景 ===
            gl.enable(glow::DEPTH_TEST);
            gl.depth_mask(true); // ✅ 关键：恢复写入深度缓冲区

            // gl.enable(glow::BLEND);
            // gl.blend_func_separate(
            //     glow::ONE,
            //     glow::ONE, // 颜色：累加所有透明颜色
            //     glow::ZERO,
            //     glow::ONE_MINUS_SRC_ALPHA, // alpha：按透明度混合
            // );

            gl.use_program(Some(self.program));

            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.program, "u_mvp").as_ref(),
                false,
                (camera.view_proj(aspect_ratio)).as_ref(),
            );
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.program, "u_model").as_ref(),
                false,
                (camera.view_matrix()).as_ref(),
            );
            gl.uniform_matrix_3_f32_slice(
                gl.get_uniform_location(self.program, "u_normal_matrix")
                    .as_ref(),
                false,
                (camera.normal_matrix()).as_ref(),
            );

            // 将光源位置转换为齐次坐标 (x,y,z,1.0)
            let light_pos_homogeneous = Vec4::new(
                light.position[0],
                light.position[1],
                light.position[2],
                1.0, // 关键：第4个分量为1.0表示点
            );

            // 应用模型变换
            let transformed_pos = camera.view_matrix() * light_pos_homogeneous;

            // 提取前三个分量 (xyz)
            let transformed_pos_xyz = [transformed_pos.x, transformed_pos.y, transformed_pos.z];

            gl.uniform_3_f32_slice(
                gl.get_uniform_location(self.program, "u_light_pos")
                    .as_ref(),
                (transformed_pos_xyz).as_ref(),
            );

            // 将光源位置转换为齐次坐标 (x,y,z,1.0)
            let camera_pos_homogeneous = Vec4::new(
                camera.position[0],
                camera.position[1],
                camera.position[2],
                1.0, // 关键：第4个分量为1.0表示点
            );

            // 应用模型变换
            let transformed_camera_pos = camera.view_matrix() * camera_pos_homogeneous;

            // 提取前三个分量 (xyz)
            let transformed_camera_pos_xyz = [
                transformed_camera_pos.x,
                transformed_camera_pos.y,
                transformed_camera_pos.z,
            ];

            gl.uniform_3_f32_slice(
                gl.get_uniform_location(self.program, "u_view_pos").as_ref(),
                (transformed_camera_pos_xyz).as_ref(),
            );

            gl.uniform_3_f32_slice(
                gl.get_uniform_location(self.program, "u_light_color")
                    .as_ref(),
                (light.color.map(|x| x * light.intensity)).as_ref(),
            );

            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "u_light_intensity")
                    .as_ref(),
                1.0,
            );

            if self.dirty {
                self.dirty = false;
            }

            // 绑定并上传缓冲
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&self.vertex3d),
                glow::DYNAMIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.element_array_buffer));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&self.indices),
                glow::DYNAMIC_DRAW,
            );

            gl.draw_elements(
                glow::TRIANGLES,
                self.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct CameraState {
    pub distance: f32,   // 距离原点的距离（保持固定）
    pub direction: Vec3, // 观察方向（通常是 unit vector）
    pub up: Vec3,        // 向上的方向
    pub scale: f32,      // 缩放比例（保持固定）
}

impl CameraState {
    pub fn new(distance: f32) -> Self {
        Self {
            distance,
            direction: Vec3::Z,
            up: Vec3::Y,
            scale: 0.5,
        }
    }
}

pub fn rotate_camera(mut camera_state: CameraState, drag_motion: Vec2) -> CameraState {
    let sensitivity = 0.005;
    let yaw = -drag_motion.x * sensitivity;    // 水平拖动 → 绕 up 旋转
    let pitch = -drag_motion.y * sensitivity;  // 垂直拖动 → 绕 right 旋转

    // 当前方向
    let dir = camera_state.direction;

    // right = 当前方向 × 当前 up
    let right = dir.cross(camera_state.up).normalize();

    // 1. pitch：绕当前 right 轴旋转（垂直）
    let pitch_quat = Quat::from_axis_angle(right, pitch);
    let rotated_dir = pitch_quat * dir;
    let rotated_up = pitch_quat * camera_state.up;

    // 2. yaw：绕当前“视角 up”旋转（水平）
    let yaw_quat = Quat::from_axis_angle(rotated_up, yaw);
    let final_dir = yaw_quat * rotated_dir;

    camera_state.direction = final_dir.normalize();
    camera_state.up = (yaw_quat * rotated_up).normalize();

    camera_state
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug, Serialize, Deserialize)]
pub struct Vertex3d {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub position: [f32; 3],
    pub z: [f32; 3],
    pub x: [f32; 3],
    pub y: [f32; 3],
    pub fov: f32,
    pub scale: f32,
}

impl Camera {
    /// 假定模型空间 == 世界空间
    pub fn new(position: [f32; 3], forward: [f32; 3], up: [f32; 3], fov: f32, scale: f32) -> Self {
        let z = Vec3::from(forward).normalize();
        let up = Vec3::from(up);
        let x = up.cross(z).normalize();
        let y = z.cross(x);

        Self {
            position,
            z: z.into(),
            x: x.into(),
            y: y.into(),
            fov,
            scale,
        }
    }

    /// 从世界空间变换到相机空间
    pub fn view_matrix(&self) -> Mat4 {
        let pos = Vec3::from(self.position);
        let center = pos + Vec3::from(self.z);
        let up = Vec3::from(self.y);

        Mat4::look_at_rh(pos, center, up)
    }

    /// 把 3D 场景投影成 2D 的视图
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        // 如果用 scale 控制的是放大倍率，可以解释为正交投影的比例因子
        let s = self.scale;

        // 你可以换成 perspective_rh(self.fov, aspect, near, far)
        Mat4::orthographic_rh(
            -s * aspect, s * aspect, // left, right
            -s, s,                   // bottom, top
            -1000.0, 1000.0          // near, far
        )
    }

    /// 相机变换矩阵 = 投影 × 视图变换
    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        self.projection_matrix(aspect) * self.view_matrix()
    }

    /// 法线矩阵：模型矩阵的 3x3 的逆转置
    pub fn normal_matrix(&self) -> Mat3 {
        Mat3::from_mat4(self.view_matrix()).inverse().transpose()
    }
}


pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}
