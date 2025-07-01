use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::shapes::sphere::Sphere;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct VisualStyle {
    pub color: Option<[f32; 3]>,
    pub opacity: f32, // 强制 0~1，默认为 1.0
    pub wireframe: bool,
    pub visible: bool,
    pub line_width: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct Interaction {
    pub clickable: bool,
    pub hoverable: bool,
    pub context_menu_enabled: bool,
    // 可扩展为事件 enum，如 Click(EventCallback)
}

// -------------------- 图元结构体 --------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Shape {
    Sphere(Sphere),
    // Cube(Cube),
    // Custom(CustomShape),
    // ...
}

// pub trait Shape {
//     fn to_mesh(&self) -> MeshData;
    
// }

pub trait ToMesh {
    fn to_mesh(&self) -> MeshData;
}

impl ToMesh for Shape {
    fn to_mesh(&self) -> MeshData {
        match self {
            Shape::Sphere(s) => s.to_mesh(),
            // Shape::Cube(c) => c.to_mesh(),
            // ...
        }
    }
}

pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub colors: Option<Vec<[f32; 4]>>,
    pub transform: Option<Mat4>, // 可选位移旋转缩放
    pub is_wireframe: bool,
}



pub trait VisualShape {
    fn style_mut(&mut self) -> &mut VisualStyle;

    fn with_color(mut self, color: [f32; 3]) -> Self
    where
        Self: Sized,
    {
        self.style_mut().color = Some(color);
        self
    }
}
