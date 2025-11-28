use glam::{Mat4, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

use crate::{
    scene::{InstanceGroups, Scene},
    shapes::{Molecules, Protein, Sphere, Stick},
};

pub trait Logger: Send + Sync + Copy {
    fn log(&self, message: impl std::fmt::Display);
    fn error(&self, message: impl std::fmt::Display);
    fn warn(&self, message: impl std::fmt::Display);
}

#[derive(Clone, Copy)]
pub struct RustLogger;

impl Logger for RustLogger {
    fn log(&self, message: impl std::fmt::Display) {
        println!("[LOG] {}", message);
    }
    fn warn(&self, message: impl std::fmt::Display) {
        eprintln!("[WARN] {}", message);
    }
    fn error(&self, message: impl std::fmt::Display) {
        eprintln!("[ERROR] {}", message);
    }
}

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
    fn interpolate(&self, other: &Self, t: f32, logger: impl Logger) -> Self;
}

// -------------------- 图元结构体 --------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Shape {
    Sphere(Sphere),
    Stick(Stick),
    Molecules(Molecules),
    Protein(Protein),
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
    fn interpolate(&self, other: &Self, t: f32, logger: impl Logger) -> Self {
        match (self, other) {
            (Shape::Sphere(a), Shape::Sphere(b)) => Shape::Sphere(a.interpolate(b, t, logger)),
            (Shape::Stick(a), Shape::Stick(b)) => Shape::Stick(a.interpolate(b, t, logger)),
            (Shape::Molecules(a), Shape::Molecules(b)) => {
                Shape::Molecules(a.interpolate(b, t, logger))
            }
            _ => self.clone(), // 如果类型不匹配，可以选择不插值或做默认处理
        }
    }
}

impl IntoInstanceGroups for Shape {
    fn to_instance_group(&self, scale: f32) -> InstanceGroups {
        let mut groups = InstanceGroups::default();

        match self {
            Shape::Sphere(s) => {
                groups.spheres.push(s.to_instance(scale));
            }
            Shape::Molecules(m) => {
                let m_groups = m.to_instance_group(scale);
                groups.merge(m_groups);
            }
            _ => {}
        }
        groups
    }
}

pub trait IntoInstanceGroups {
    fn to_instance_group(&self, scale: f32) -> InstanceGroups;
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
            Shape::Protein(s) => s.to_mesh(scale),
            Shape::Qudrate => todo!(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub colors: Option<Vec<Vec4>>,
    pub transform: Option<Mat4>, // 可选位移旋转缩放
    pub is_wireframe: bool,
}

impl MeshData {
    /// Append another MeshData into this one.
    pub fn append(&mut self, other: &MeshData) {
        let base = self.vertices.len() as u32;

        // append vertices
        self.vertices.extend(&other.vertices);

        // append normals
        self.normals.extend(&other.normals);

        // append colors
        if let Some(ref mut my_colors) = self.colors {
            if let Some(ref other_colors) = other.colors {
                my_colors.extend(other_colors);
            }
        } else if let Some(ref other_colors) = other.colors {
            self.colors = Some(other_colors.clone());
        }

        // append indices with offset
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }
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
    pub smooth: bool,
}

use half::f16;
use serde::{Deserializer, Serializer};
pub mod vec_f16 {
    use super::*;

    pub fn serialize<S, T>(v: &T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoF32Slice,
    {
        let bits: Vec<u16> = v
            .as_f32_slice()
            .iter()
            .map(|x| f16::from_f32(*x).to_bits())
            .collect();
        bits.serialize(s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromF32Slice,
    {
        let bits: Vec<u16> = Vec::<u16>::deserialize(d)?;
        let floats: Vec<f32> = bits.iter().map(|b| f16::from_bits(*b).to_f32()).collect();
        Ok(T::from_f32_slice(&floats))
    }

    // ---- Vec<Vec2> / Vec<Vec3> ----

    pub mod vec {
        use super::*;

        pub fn serialize<S, T>(v: &Vec<T>, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: IntoF32Slice,
        {
            let out: Vec<Vec<u16>> = v
                .iter()
                .map(|e| {
                    e.as_f32_slice()
                        .iter()
                        .map(|x| f16::from_f32(*x).to_bits())
                        .collect()
                })
                .collect();
            out.serialize(s)
        }

        pub fn deserialize<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
        where
            D: Deserializer<'de>,
            T: FromF32Slice,
        {
            let arr: Vec<Vec<u16>> = Vec::<Vec<u16>>::deserialize(d)?;
            let out = arr
                .into_iter()
                .map(|v| {
                    let floats: Vec<f32> = v.iter().map(|b| f16::from_bits(*b).to_f32()).collect();
                    T::from_f32_slice(&floats)
                })
                .collect();
            Ok(out)
        }
    }
}

// ====== Trait definitions ======

pub trait IntoF32Slice {
    fn as_f32_slice(&self) -> &[f32];
}

pub trait FromF32Slice {
    fn from_f32_slice(v: &[f32]) -> Self;
}

// -------- Vec2 --------

impl IntoF32Slice for Vec2 {
    fn as_f32_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self as *const Vec2 as *const f32, 2) }
    }
}
impl FromF32Slice for Vec2 {
    fn from_f32_slice(v: &[f32]) -> Self {
        Self::new(v[0], v[1])
    }
}

// -------- Vec3 --------

impl IntoF32Slice for Vec3 {
    fn as_f32_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self as *const Vec3 as *const f32, 3) }
    }
}
impl FromF32Slice for Vec3 {
    fn from_f32_slice(v: &[f32]) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}

// -------- Vec4 --------

impl IntoF32Slice for Vec4 {
    fn as_f32_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self as *const Vec4 as *const f32, 4) }
    }
}
impl FromF32Slice for Vec4 {
    fn from_f32_slice(v: &[f32]) -> Self {
        Self::new(v[0], v[1], v[2], v[3])
    }
}
