use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::shapes::{molecules::Molecules, sphere::Sphere, stick::Stick};

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct VisualStyle {
    pub color: Option<[f32; 3]>,
    pub opacity: f32,
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
    Stick(Stick),
    Molecules(Molecules),
    // Custom(CustomShape),
    // ...
}

pub trait ToMesh {
    fn to_mesh(&self, scale: f32) -> MeshData;
}

impl ToMesh for Shape {
    fn to_mesh(&self, scale: f32) -> MeshData {
        match self {
            Shape::Sphere(s) => s.to_mesh(scale),
            Shape::Stick(s) => s.to_mesh(scale),
            Shape::Molecules(s) => s.to_mesh(scale),
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

    fn color(mut self, color: [f32; 3]) -> Self
    where
        Self: Sized,
    {
        self.style_mut().color = Some(color);
        self
    }

    fn color_rgba(mut self, color: [f32; 4]) -> Self
    where
        Self: Sized,
    {
        self.style_mut().color = Some(color[0..3].try_into().unwrap());
        self.style_mut().opacity = color[3];

        self
    }

    fn opacity(mut self, opacity: f32) -> Self
    where
        Self: Sized,
    {
        self.style_mut().opacity = opacity;
        self
    }
}
