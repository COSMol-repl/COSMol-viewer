use crate::utils::Logger;
use glam::Mat3;
use glam::Mat4;
use std::collections::HashMap;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{
    Shape,
    shader::CameraState,
    utils::{self, Interpolatable, IntoInstanceGroups, ToMesh},
};

pub enum Instance {
    Sphere(SphereInstance),
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct SphereInstance {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

impl SphereInstance {
    pub fn new(position: [f32; 3], radius: f32, color: [f32; 4]) -> Self {
        Self {
            position,
            radius,
            color,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct StickInstance {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

impl StickInstance {
    pub fn new(start: [f32; 3], end: [f32; 3], radius: f32, color: [f32; 4]) -> Self {
        Self {
            start,
            end,
            radius,
            color,
        }
    }
}

#[derive(Clone, Default)]
pub struct InstanceGroups {
    pub spheres: Vec<SphereInstance>,
    pub sticks: Vec<StickInstance>,
}

impl InstanceGroups {
    pub fn merge(&mut self, other: InstanceGroups) {
        self.spheres.extend(other.spheres);
        self.sticks.extend(other.sticks);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Scene {
    pub background_color: [f32; 3],
    pub camera_state: CameraState,
    pub named_shapes: HashMap<String, Shape>,
    pub unnamed_shapes: Vec<Shape>,
    pub scale: f32,
    pub viewport: Option<[usize; 2]>,
    pub scene_center: [f32; 3],
}

impl Scene {
    pub fn _get_meshes(&self) -> Vec<utils::MeshData> {
        self.named_shapes
            .values()
            .chain(self.unnamed_shapes.iter())
            .map(|s| s.to_mesh(self.scale))
            .collect()
    }
    pub fn get_instances_grouped(&self) -> InstanceGroups {
        let mut groups = InstanceGroups::default();

        for shape in self.named_shapes.values().chain(self.unnamed_shapes.iter()) {
            let shape_groups = shape.to_instance_group(self.scale);
            groups.merge(shape_groups);
        }

        groups
    }

    pub fn new() -> Self {
        Scene {
            background_color: [1.0, 1.0, 1.0],
            camera_state: CameraState::new(35.0),
            named_shapes: HashMap::new(),
            unnamed_shapes: Vec::new(),
            scale: 1.0,
            viewport: None,
            scene_center: [0.0, 0.0, 0.0],
        }
    }

    pub fn recenter(&mut self, center: [f32; 3]) {
        self.scene_center = center;
    }

    pub fn scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn add_shape<S: Into<Shape>>(&mut self, shape: S, id: Option<&str>) {
        let shape = shape.into();
        if let Some(id) = id {
            self.named_shapes.insert(id.into(), shape);
        } else {
            self.unnamed_shapes.push(shape);
        }
    }

    pub fn update_shape<S: Into<Shape>>(&mut self, id: &str, shape: S) {
        let shape = shape.into();
        if let Some(existing_shape) = self.named_shapes.get_mut(id) {
            *existing_shape = shape;
        } else {
            panic!("Shape with ID '{}' not found", id);
        }
    }

    pub fn delete_shape(&mut self, id: &str) {
        if self.named_shapes.remove(id).is_none() {
            panic!("Sphere with ID '{}' not found", id);
        }
    }

    pub fn set_background_color(&mut self, background_color: [f32; 3]) {
        self.background_color = background_color;
    }

    pub fn use_black_background(&mut self) {
        self.background_color = [0.0, 0.0, 0.0];
    }

    /// === u_model ===
    /// 把整个模型平移，使得 scene_center 成为原点
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_translation(-Vec3::from(self.scene_center) * self.scale)
    }

    /// === u_normal_matrix ===
    /// 模型矩阵逆转置（仅对 3x3）
    pub fn normal_matrix(&self) -> Mat3 {
        Mat3::from_mat4(self.model_matrix()).inverse().transpose()
    }
}

impl Interpolatable for Scene {
    fn interpolate(&self, other: &Self, t: f32, logger: impl Logger) -> Self {
        let named_shapes = self
            .named_shapes
            .iter()
            .map(|(k, v)| {
                let other_shape = &other.named_shapes[k];
                (k.clone(), v.interpolate(other_shape, t, logger))
            })
            .collect();

        let unnamed_shapes = self
            .unnamed_shapes
            .iter()
            .zip(&other.unnamed_shapes)
            .map(|(a, b)| a.interpolate(b, t, logger))
            .collect();

        let scene_center =
            Vec3::from(self.scene_center) * (1.0 - t) + Vec3::from(other.scene_center) * t;

        Self {
            background_color: self.background_color,
            camera_state: self.camera_state, // 可以单独插值
            named_shapes,
            unnamed_shapes,
            scale: self.scale * (1.0 - t) + other.scale * t,
            viewport: self.viewport,
            scene_center: [scene_center.x, scene_center.y, scene_center.z],
        }
    }
}
