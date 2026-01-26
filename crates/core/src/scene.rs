use crate::utils::InstanceGroups;
use crate::utils::Logger;
use glam::Mat3;
use glam::Mat4;
use std::collections::HashMap;
use thiserror::Error;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{
    Shape,
    shader::CameraState,
    utils::{self, Interpolatable, IntoInstanceGroups, ToMesh},
};

// pub enum Instance {
//     Sphere(SphereInstance),
// }

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

#[derive(Error, Debug)]
pub enum SceneError {
    #[error("Shape with ID '{0}' not found")]
    ShapeNotFound(String),
    #[error("Failed to parse scene from JSON")]
    ParseError(#[from] serde_json::Error),
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            background_color: [1.0, 1.0, 1.0],
            camera_state: CameraState::new(35.0),
            named_shapes: HashMap::new(),
            unnamed_shapes: Vec::new(),
            scale: 1.0,
            viewport: None,
            scene_center: [0.0, 0.0, 0.0],
        }
    }
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
        self.named_shapes
            .values()
            .chain(self.unnamed_shapes.iter())
            .map(|s| s.to_instance_group(self.scale))
            .fold(InstanceGroups::default(), |mut acc, g| {
                acc.merge(g);
                acc
            })
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn recenter(&mut self, center: [f32; 3]) {
        self.scene_center = center;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn add_shape_with_id<S: Into<Shape>>(&mut self, id: impl Into<String>, shape: S) {
        self.named_shapes.insert(id.into(), shape.into());
    }

    pub fn add_shape<S: Into<Shape>>(&mut self, shape: S) {
        self.unnamed_shapes.push(shape.into());
    }

    pub fn replace_shape<S: Into<Shape>>(&mut self, id: &str, shape: S) -> Result<(), SceneError> {
        let shape = shape.into();
        if let Some(existing_shape) = self.named_shapes.get_mut(id) {
            *existing_shape = shape;
            Ok(())
        } else {
            Err(SceneError::ShapeNotFound(id.to_string()))
        }
    }

    pub fn remove_shape(&mut self, id: &str) -> Result<(), SceneError> {
        if self.named_shapes.remove(id).is_none() {
            Err(SceneError::ShapeNotFound(id.to_string()))
        } else {
            Ok(())
        }
    }

    pub fn set_background_color(&mut self, background_color: [f32; 3]) {
        self.background_color = background_color;
    }

    pub fn use_black_background(&mut self) {
        self.background_color = [0.0, 0.0, 0.0];
    }

    /// === u_model ===
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_translation(-Vec3::from(self.scene_center) * self.scale)
    }

    /// === u_normal_matrix ===
    pub fn normal_matrix(&self) -> Mat3 {
        Mat3::from_mat4(self.model_matrix()).inverse().transpose()
    }

    pub fn merge_shapes(&mut self, other: &Self) {
        self.unnamed_shapes
            .extend(other.named_shapes.iter().map(|(_k, v)| v.clone()));
        self.unnamed_shapes.extend(other.unnamed_shapes.clone());
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Animation {
    pub static_scene: Option<Scene>,
    pub frames: Vec<Scene>,
    pub interval: u64,
    pub loops: i64, // -1 = infinite
    pub smooth: bool,
}

impl Animation {
    pub fn new(interval: f32, loops: i64, smooth: bool) -> Self {
        Self {
            static_scene: None,
            frames: Vec::new(),
            interval: (interval * 1000.0) as u64,
            loops,
            smooth,
        }
    }

    pub fn add_frame(&mut self, frame: Scene) {
        self.frames.push(frame);
    }

    pub fn set_static_scene(&mut self, scene: Scene) {
        self.static_scene = Some(scene);
    }
}
