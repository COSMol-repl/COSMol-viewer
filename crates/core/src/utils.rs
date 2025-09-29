use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::{
    scene::{Instance, InstanceGroups, Scene, SphereInstance},
    shapes::{Molecules, Sphere, Stick},
};

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

pub trait Interpolatable {
    /// t ∈ [0.0, 1.0]，返回两个实例之间的插值
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

// -------------------- 图元结构体 --------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Shape {
    Sphere(Sphere),
    Stick(Stick),
    Molecules(Molecules),
    Qudrate, // Custom(CustomShape),
             // ...
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShapeKind {
    Sphere,
    Stick,
}

pub struct InstanceData {
    pub position: [f32; 3],
    pub scale: f32, // 比如 Sphere 半径或 Cylinder 长度
    pub color: [f32; 4],
    pub extra: Option<[f32; 3]>, // 比如 Cylinder 要方向向量
}

impl Interpolatable for Shape {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        match (self, other) {
            (Shape::Sphere(a), Shape::Sphere(b)) => Shape::Sphere(a.interpolate(b, t)),
            (Shape::Stick(a), Shape::Stick(b)) => Shape::Stick(a.interpolate(b, t)),
            (Shape::Molecules(a), Shape::Molecules(b)) => Shape::Molecules(a.interpolate(b, t)),
            _ => self.clone(), // 如果类型不匹配，可以选择不插值或做默认处理
        }
    }
}

pub trait IntoInstanceGroups {
    fn to_instance_group(&self, scale: f32) -> InstanceGroups;
}

impl IntoInstanceGroups for Shape {
    fn to_instance_group(&self, scale: f32) -> InstanceGroups {
        let mut groups = InstanceGroups::default();

        match self {
            Shape::Sphere(s) => {
                let base_color = s.style.color.unwrap_or([1.0, 1.0, 1.0]);
                let alpha = s.style.opacity.clamp(0.0, 1.0);
                let color = [base_color[0], base_color[1], base_color[2], alpha];

                groups
                    .spheres
                    .push(SphereInstance::new(s.center.map(|x| x * scale), s.radius * scale, color));
            }
            Shape::Molecules(m) => {
                let m_groups = m.to_meta_shape_group();
                groups.merge(m_groups);
            }
            _ => {},
        }

        groups
    }
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
            Shape::Qudrate => todo!(),
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Frames {
    pub frames: Vec<Scene>,
    pub interval: u64,
    pub loops: i64, // -1 = infinite
}
