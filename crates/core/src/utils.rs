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
    pub color: Option<Vec3>,
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
        self.style_mut().color = Some(color.into());
        self
    }

    fn color_rgba(mut self, color: [f32; 4]) -> Self
    where
        Self: Sized,
    {
        self.style_mut().color = Some(Vec3::new(color[0], color[1], color[2]));
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
        let bits: Vec<u32> = v.as_f32_slice().iter().map(|x| x.to_bits()).collect();
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

pub mod vec_f16_scaled {
    use super::*;

    // 你可以选择固定系数，或者运行时传
    pub const SCALE: f32 = 3000.0; // 根据你的数据调整，1000~5000 都很常见

    pub fn serialize<S, T>(v: &T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoF32Slice,
    {
        let bits: Vec<u16> = v
            .as_f32_slice()
            .iter()
            .map(|x| f16::from_f32(x * SCALE).to_bits())
            .collect();
        bits.serialize(s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromF32Slice,
    {
        let bits: Vec<u16> = Vec::<u16>::deserialize(d)?;
        let floats: Vec<f32> = bits
            .iter()
            .map(|b| f16::from_bits(*b).to_f32() / SCALE)
            .collect();
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
            let bits: Vec<Vec<u16>> = v
                .iter()
                .map(|v| {
                    let floats: Vec<f32> = v.as_f32_slice().iter().map(|x| x * SCALE).collect();
                    floats
                        .into_iter()
                        .map(|x| f16::from_f32(x).to_bits())
                        .collect()
                })
                .collect();
            bits.serialize(s)
        }

        pub fn deserialize<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
        where
            D: Deserializer<'de>,
            T: FromF32Slice,
        {
            let bits: Vec<Vec<u16>> = Vec::<Vec<u16>>::deserialize(d)?;
            let out = bits
                .into_iter()
                .map(|v| {
                    let floats: Vec<f32> = v
                        .iter()
                        .map(|b| f16::from_bits(*b).to_f32() / SCALE)
                        .collect();
                    T::from_f32_slice(&floats)
                })
                .collect();
            Ok(out)
        }
    }
}

// pub mod vec_binary {
//     use super::*;
//     use base64::{Engine, engine::general_purpose::STANDARD as b64};
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};

//     // -------------------------------------------------------
//     // 单个 Vec2 / Vec3：无损（f32 -> u32 bits）
//     // -------------------------------------------------------
//     pub fn serialize<S, T>(v: &T, s: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//         T: IntoF32Slice,
//     {
//         let bits: Vec<u32> = v.as_f32_slice().iter().map(|x| x.to_bits()).collect();
//         bits.serialize(s)
//     }

//     pub fn deserialize<'de, D, T>(d: D) -> Result<T, D::Error>
//     where
//         D: Deserializer<'de>,
//         T: FromF32Slice,
//     {
//         let bits: Vec<u32> = Vec::<u32>::deserialize(d)?;
//         let floats: Vec<f32> = bits.iter().map(|b| f32::from_bits(*b)).collect();
//         Ok(T::from_f32_slice(&floats))
//     }

//     // ==========================================================
//     // Vec<T>：base64 无损存储（高压缩），不使用 f16
//     // ==========================================================
//     pub mod vec {
//         use super::*;

//         pub fn serialize<S, T>(v: &Vec<T>, s: S) -> Result<S::Ok, S::Error>
//         where
//             S: Serializer,
//             T: IntoF32Slice,
//         {
//             // 1. flatten 成连续 f32
//             let mut floats = Vec::<f32>::new();
//             for item in v {
//                 floats.extend_from_slice(item.as_f32_slice());
//             }

//             // 2. f32 -> u32 bits -> bytes
//             let mut bytes = Vec::<u8>::new();
//             for f in floats {
//                 bytes.extend_from_slice(&f.to_bits().to_le_bytes());
//             }

//             // 3. base64
//             let encoded = b64.encode(bytes);

//             s.serialize_str(&encoded)
//         }

//         pub fn deserialize<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
//         where
//             D: Deserializer<'de>,
//             T: FromF32Slice,
//         {
//             // 1. base64 decode
//             let encoded = String::deserialize(d)?;
//             let bytes = b64
//                 .decode(encoded.as_bytes())
//                 .map_err(serde::de::Error::custom)?;

//             if bytes.len() % 4 != 0 {
//                 return Err(serde::de::Error::custom("byte length not aligned"));
//             }

//             // 2. bytes -> u32 bits -> f32
//             let mut floats = Vec::<f32>::new();
//             for chunk in bytes.chunks_exact(4) {
//                 let bits = u32::from_le_bytes(chunk.try_into().unwrap());
//                 floats.push(f32::from_bits(bits));
//             }

//             // 3. 按 T::dim() 重建 T
//             let dim = T::dim();
//             if floats.len() % dim != 0 {
//                 return Err(serde::de::Error::custom("dimension mismatch"));
//             }

//             let mut out = Vec::<T>::new();
//             for chunk in floats.chunks(dim) {
//                 out.push(T::from_f32_slice(chunk));
//             }

//             Ok(out)
//         }
//     }
// }

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

use base64::{Engine, engine::general_purpose::STANDARD as B64};
use bytemuck::Pod;

pub mod vec_compact {
    use super::*;

    pub fn serialize<T, S>(data: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Pod,
    {
        if data.is_empty() {
            return serializer.serialize_str("");
        }

        // 直接 cast 成字节流，最快方式（零拷贝！）
        let bytes = bytemuck::cast_slice(data);
        let encoded = B64.encode(bytes);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Pod,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            return Ok(Vec::new());
        }

        let bytes = B64.decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() % std::mem::size_of::<T>() != 0 {
            return Err(serde::de::Error::custom("length not aligned to type size"));
        }

        // 安全 + 零拷贝
        Ok(bytemuck::pod_collect_to_vec(&bytes))
    }
}

pub fn serialize_bits<S>(v: &Vec3, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bits: [u32; 3] = [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()];
    bits.serialize(serializer)
}

pub fn deserialize_bits<'de, D>(deserializer: D) -> Result<Vec3, D::Error>
where
    D: Deserializer<'de>,
{
    let bits: [u32; 3] = Deserialize::deserialize(deserializer)?;
    Ok(Vec3::new(
        f32::from_bits(bits[0]),
        f32::from_bits(bits[1]),
        f32::from_bits(bits[2]),
    ))
}
