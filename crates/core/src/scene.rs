use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{shader::CameraState, utils::{self, ToMesh}, Shape};


#[derive(Deserialize, Serialize, Clone)]
pub struct Scene {
    pub background_color: [f32; 3],
    pub camera_state: CameraState,
    pub named_shapes: HashMap<String, Shape>,
    pub unnamed_shapes: Vec<Shape>,
    pub scale: f32,
}

impl Scene {
    pub fn _get_meshes(&self) -> Vec<utils::MeshData> {
        self.named_shapes
            .values()
            .chain(self.unnamed_shapes.iter())
            .map(|s| s.to_mesh(self.scale))
            .collect()
    }

    pub fn new() -> Self {
        Scene {
            background_color: [1.0, 1.0, 1.0],
            camera_state: CameraState::new(1.0),
            named_shapes: HashMap::new(),
            unnamed_shapes: Vec::new(),
            scale: 1.0,
        }
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

    pub fn delete_shape(&mut self, id: &str) {
        if self.named_shapes.remove(id).is_none() {
            panic!("Sphere with ID '{}' not found", id);
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
}