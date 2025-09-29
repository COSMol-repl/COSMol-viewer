use glam::Mat3;
use glam::Mat4;
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use wasm_bindgen_futures::js_sys::WebAssembly::Instance;

use eframe::{
    egui::{self, Vec2, mutex::Mutex},
    egui_glow, glow,
};
use glam::{Quat, Vec3};

use crate::Scene;
use crate::scene::InstanceGroups;
use crate::scene::SphereInstance;
use crate::shapes::Sphere;
use crate::utils::Frames;
use crate::utils::Interpolatable;

pub struct Canvas {
    shader: Arc<Mutex<Shader>>,
    camera_state: CameraState,
    last_frame_time: std::time::Instant,
    frames: Option<Frames>,
    current_frame: usize,
    current_loop: usize,
}

pub fn interpolate_frames(frame_a: &Scene, frame_b: &Scene, t: f32) -> Scene {
    frame_a.interpolate(frame_b, t)
}

impl Canvas {
    pub fn new<'a>(gl: Arc<eframe::glow::Context>, scene: Scene) -> Option<Self> {
        Some(Self {
            shader: Arc::new(Mutex::new(Shader::new(&gl, scene)?)),
            camera_state: CameraState::new(1.0),
            last_frame_time: std::time::Instant::now(),
            current_frame: 0,
            current_loop: 0,
            frames: None,
        })
    }

    pub fn new_play<'a>(gl: Arc<eframe::glow::Context>, frames: Frames) -> Option<Self> {
        Some(Self {
            shader: Arc::new(Mutex::new(Shader::new(&gl, frames.frames[0].clone())?)),
            camera_state: CameraState::new(1.0),
            last_frame_time: Instant::now(),
            current_frame: 0,
            current_loop: 0,
            frames: Some(frames),
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

        if let Some(frames) = &mut self.frames {
            if (self.current_loop < frames.loops as usize) {
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_frame_time);
                let t = (elapsed.as_secs_f32() / (frames.interval as f32 / 1000.0)).clamp(0.0, 1.0);
                let total_frames = frames.frames.len();

                let frame_a_index = self.current_frame;
                let frame_b_index = if self.current_frame == total_frames - 1 && frames.loops != -1
                {
                    self.current_frame // 不跳回 0
                } else {
                    (self.current_frame + 1) % total_frames
                };

                let frame_a = &frames.frames[frame_a_index];
                let frame_b = &frames.frames[frame_b_index];

                let interp_frame = interpolate_frames(frame_a, frame_b, t);
                self.shader.lock().update_scene(interp_frame);

                if elapsed.as_millis() >= frames.interval as u128 {
                    self.current_frame = frame_b_index;
                    self.last_frame_time = now;
                    if self.current_frame == 0 && frames.loops != -1 {
                        self.current_loop += 1;
                        if self.current_loop >= frames.loops as usize {
                            self.current_frame = frames.frames.len() - 1; // 停在最后一帧
                        }
                    }
                }
                ui.ctx().request_repaint();
            }
        }

        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);

        // 正值表示向上滚动，通常是“缩小”，负值是放大
        if scroll_delta != 0.0 {
            self.camera_state.scale *= (1.0 + scroll_delta * 0.001).clamp(0.1, 10.0);
        }

        if response.dragged() {
            self.camera_state = rotate_camera(self.camera_state, response.drag_motion());
        }

        // Clone locals so we can move them into the paint callback:
        let shader = self.shader.clone();

        let aspect_ratio = rect.width() / rect.height();
        let camera_state = self.camera_state.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            shader
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
    }
}

struct Shader {
    program: glow::Program,
    program_bg: glow::Program,
    program_sphere: glow::Program,
    vao_mesh: glow::VertexArray,
    vao_sphere: glow::VertexArray,
    vertex3d: Vec<Vertex3d>,
    indices: Vec<u32>,
    sphere_index_count: usize,
    background_color: [f32; 3],
    vertex_buffer: glow::Buffer,
    ebo: glow::Buffer,
    sphere_buffer: glow::Buffer,
    instance_groups: Option<InstanceGroups>,
}

#[expect(unsafe_code)] // we need unsafe code to use glow
impl Shader {
    fn new(gl: &glow::Context, scene: Scene) -> Option<Self> {
        use glow::HasContext as _;

        let shader_version = egui_glow::ShaderVersion::get(gl);
        let background_color = scene.background_color;
        let default_color = [1.0, 1.0, 1.0, 1.0];

        unsafe {
            // =========================
            // 1. Create shader programs
            // =========================
            let program_bg = gl.create_program().expect("Cannot create program");
            let program = gl.create_program().expect("Cannot create program");
            let program_sphere = gl.create_program().expect("Cannot create program");

            if !shader_version.is_new_shader_interface() {
                println!(
                    "Custom 3D painting hasn't been ported to {:?}",
                    shader_version
                );
                return None;
            }

            // =========================
            // 2. Load shader sources
            // =========================
            let (vertex_shader, fragment_shader) = (
                include_str!("./vertex.glsl"),
                include_str!("./fragment.glsl"),
            );

            let (vertex_shader_bg, fragment_shader_bg) = (
                include_str!("./bg_vertex.glsl"),
                include_str!("./bg_fragment.glsl"),
            );

            let vertex_sphere_shader = include_str!("./vertex_sphere.glsl");

            let shader = [
                (glow::VERTEX_SHADER, vertex_shader),
                (glow::FRAGMENT_SHADER, fragment_shader),
            ];

            let shader_bg = [
                (glow::VERTEX_SHADER, vertex_shader_bg),
                (glow::FRAGMENT_SHADER, fragment_shader_bg),
            ];

            let shader_sphere = [
                (glow::VERTEX_SHADER, vertex_sphere_shader),
                (glow::FRAGMENT_SHADER, fragment_shader),
            ];

            println!("shader_version = {:?}", shader_version);

            // =========================
            // 3. Compile and link main shader
            // =========================
            let shaders: Vec<_> = shader
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

            // =========================
            // 4. Compile and link background shader
            // =========================
            let shaders_bg: Vec<_> = shader_bg
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader_bg");
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

            // =========================
            // 5. Compile and link sphere shader
            // =========================
            let shaders_sphere: Vec<_> = shader_sphere
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader_sphere");
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
                        "Failed to compile custom_3d_glow_sphere {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );

                    gl.attach_shader(program_sphere, shader);
                    shader
                })
                .collect();

            gl.link_program(program_sphere);
            assert!(
                gl.get_program_link_status(program_sphere),
                "{}",
                gl.get_program_info_log(program_sphere)
            );

            for shader in shaders_sphere {
                gl.detach_shader(program_sphere, shader);
                gl.delete_shader(shader);
            }

            // =========================
            // 6. Generate sphere mesh template
            // =========================
            let template_sphere = Sphere::get_or_generate_sphere_mesh_template(32);

            let vertex3d_sphere: Vec<Vertex3d> = template_sphere
                .vertices
                .iter()
                .enumerate()
                .map(|(i, pos)| Vertex3d {
                    position: *pos,
                    normal: template_sphere.normals[i],
                    color: default_color,
                })
                .collect();

            let indices_sphere: Vec<u32> = template_sphere.indices.clone();

            // =========================
            // 7. Create buffers
            // =========================
            let vertex_buffer = gl.create_buffer().expect("Cannot create vertex buffer");
            let ebo = gl.create_buffer().expect("Cannot create element buffer");
            let sphere_buffer = gl
                .create_buffer()
                .expect("Cannot create sphere instance buffer");

            let sphere_vertex_buffer = gl
                .create_buffer()
                .expect("Cannot create sphere vertex buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(sphere_vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertex3d_sphere),
                glow::STATIC_DRAW,
            );

            let sphere_ebo = gl
                .create_buffer()
                .expect("Cannot create sphere element buffer");
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(sphere_ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&indices_sphere),
                glow::STATIC_DRAW,
            );

            // =========================
            // 8. Setup VAO for mesh
            // =========================
            let vao_mesh = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vao_mesh));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            
            let pos_loc = gl.get_attrib_location(program, "a_position").unwrap();
            let normal_loc = gl.get_attrib_location(program, "a_normal").unwrap();
            let color_loc = gl.get_attrib_location(program, "a_color").unwrap();
            
            let stride_vertex = std::mem::size_of::<Vertex3d>() as i32;

            gl.enable_vertex_attrib_array(pos_loc);
            gl.vertex_attrib_pointer_f32(pos_loc, 3, glow::FLOAT, false, stride_vertex, 0);

            gl.enable_vertex_attrib_array(normal_loc);
            gl.vertex_attrib_pointer_f32(normal_loc, 3, glow::FLOAT, false, stride_vertex, 3 * 4);

            gl.enable_vertex_attrib_array(color_loc);
            gl.vertex_attrib_pointer_f32(color_loc, 4, glow::FLOAT, false, stride_vertex, 6 * 4);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

            // =========================
            // 9. Setup VAO for instanced spheres
            // =========================
            gl.bind_attrib_location(program_sphere, 0, "a_position");
            gl.bind_attrib_location(program_sphere, 1, "a_normal");
            gl.bind_attrib_location(program_sphere, 2, "i_position");
            gl.bind_attrib_location(program_sphere, 3, "i_radius");
            gl.bind_attrib_location(program_sphere, 4, "i_color");

            let vao_sphere = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vao_sphere));

            // per-vertex attributes
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(sphere_vertex_buffer));
            gl.enable_vertex_attrib_array(0); // a_position
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride_vertex, 0);
            gl.vertex_attrib_divisor(0, 0);

            gl.enable_vertex_attrib_array(1); // a_normal
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride_vertex, 3 * 4);
            gl.vertex_attrib_divisor(1, 0);

            // per-instance attributes
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(sphere_buffer));
            let stride_instance = std::mem::size_of::<SphereInstance>() as i32;

            gl.enable_vertex_attrib_array(2); // i_position
            gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, stride_instance, 0);
            gl.vertex_attrib_divisor(2, 1);

            gl.enable_vertex_attrib_array(3); // i_radius
            gl.vertex_attrib_pointer_f32(3, 1, glow::FLOAT, false, stride_instance, 3 * 4);
            gl.vertex_attrib_divisor(3, 1);

            gl.enable_vertex_attrib_array(4); // i_color
            gl.vertex_attrib_pointer_f32(4, 4, glow::FLOAT, false, stride_instance, 4 * 4);
            gl.vertex_attrib_divisor(4, 1);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(sphere_ebo));
            gl.bind_vertex_array(None);

            gl.use_program(Some(program));

            // =========================
            // 10. Create shader instance struct
            // =========================
            let mut shader_instance = Self {
                program,
                program_bg,
                program_sphere,
                vertex3d: vec![],
                indices: vec![],
                vao_mesh,
                vao_sphere,
                background_color,
                vertex_buffer,
                ebo,
                sphere_buffer,
                instance_groups: None,
                sphere_index_count: indices_sphere.len(),
            };

            // =========================
            // 11. Update scene data
            // =========================
            shader_instance.update_scene(scene);

            Some(shader_instance)
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

        self.instance_groups = Some(scene_data.get_instances_grouped());
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
            direction: [1.0, -1.0, -2.0],
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
            gl.enable(glow::MULTISAMPLE); // 开启多重采样

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

            // 将光源位置转换为齐次坐标 (x,y,z,1.0)
            let light_pos_homogeneous = Vec4::new(
                -light.direction[0],
                -light.direction[1],
                -light.direction[2],
                1.0, // 关键：第4个分量为1.0表示点
            );

            // 应用模型变换
            let transformed_light_pos = light_pos_homogeneous;

            // 提取前三个分量 (xyz)
            let transformed_light_pos_xyz = [
                transformed_light_pos.x,
                transformed_light_pos.y,
                transformed_light_pos.z,
            ];

            // 将摄像机位置转换为齐次坐标 (x,y,z,1.0)
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

            gl.uniform_3_f32_slice(
                gl.get_uniform_location(self.program, "u_light_pos")
                    .as_ref(),
                (transformed_light_pos_xyz).as_ref(),
            );

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

            // 绑定并上传缓冲
            gl.bind_vertex_array(Some(self.vao_mesh));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&self.vertex3d),
                glow::DYNAMIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
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

            if let Some(instance_groups) = &self.instance_groups {
                println!("Drawing {} spheres", instance_groups.spheres.len());
                gl.use_program(Some(self.program_sphere));
                gl.uniform_matrix_4_f32_slice(
                    gl.get_uniform_location(self.program_sphere, "u_mvp")
                        .as_ref(),
                    false,
                    (camera.view_proj(aspect_ratio)).as_ref(),
                );
                gl.uniform_matrix_4_f32_slice(
                    gl.get_uniform_location(self.program_sphere, "u_model")
                        .as_ref(),
                    false,
                    (camera.view_matrix()).as_ref(),
                );
                gl.uniform_matrix_3_f32_slice(
                    gl.get_uniform_location(self.program_sphere, "u_normal_matrix")
                        .as_ref(),
                    false,
                    (camera.normal_matrix()).as_ref(),
                );

                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(self.program_sphere, "u_light_pos")
                        .as_ref(),
                    (transformed_light_pos_xyz).as_ref(),
                );

                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(self.program_sphere, "u_view_pos")
                        .as_ref(),
                    (transformed_camera_pos_xyz).as_ref(),
                );

                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(self.program_sphere, "u_light_color")
                        .as_ref(),
                    (light.color.map(|x| x * light.intensity)).as_ref(),
                );

                gl.uniform_1_f32(
                    gl.get_uniform_location(self.program_sphere, "u_light_intensity")
                        .as_ref(),
                    1.0,
                );

                gl.bind_vertex_array(Some(self.vao_sphere));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.sphere_buffer));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&instance_groups.spheres),
                    glow::DYNAMIC_DRAW,
                );

                gl.draw_elements_instanced(
                    glow::TRIANGLES,
                    self.sphere_index_count as i32,
                    glow::UNSIGNED_INT,
                    0,
                    instance_groups.spheres.len() as i32,
                );
            }
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
    let yaw = -drag_motion.x * sensitivity; // 水平拖动 → 绕 up 旋转
    let pitch = -drag_motion.y * sensitivity; // 垂直拖动 → 绕 right 旋转

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
            -s * aspect,
            s * aspect, // left, right
            -s,
            s, // bottom, top
            -1000.0,
            1000.0, // near, far
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
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}
