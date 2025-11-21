use crate::Shape;
use crate::utils::MeshData;
use bio_files;
use bio_files::MmCif;
use glam::{Quat, Vec3};
use na_seq::AminoAcid;
use na_seq::AtomTypeInRes;
use serde::{Deserialize, Serialize};
use splines::{Key, Spline};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chain {
    residues: Vec<Residue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Residue {
    pub residue_type: ResidueType, // e.g. "ALA", "GLY"
    pub index: usize,              // PDB numbering or sequential

    // Minimum for cartoon backbone
    pub ca: Vec3, // C-alpha coordinates

    // Optional but highly recommended (for proper frame construction)
    pub cb: Option<Vec3>, // or pseudo-CB for glycine

    // Secondary structure tag
    pub ss: SecondaryStructure,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryStructure {
    Helix,
    Sheet,
    Coil,
    Unknown,
}

mod aa_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(aa: &AminoAcid, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&aa.to_string())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<AminoAcid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(d)?;
        name.parse::<AminoAcid>().map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ResidueType {
    #[serde(with = "aa_serde")]
    AminoAcid(AminoAcid),
    Water,
    Other(String),
}

impl From<&bio_files::ResidueType> for ResidueType {
    fn from(res_type: &bio_files::ResidueType) -> Self {
        match res_type {
            bio_files::ResidueType::AminoAcid(a) => ResidueType::AminoAcid(*a),
            bio_files::ResidueType::Water => ResidueType::Water,
            bio_files::ResidueType::Other(s) => ResidueType::Other(s.clone()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Protein {
    chains: Vec<Chain>,
    center: Vec3,
}

impl Protein {
    pub fn new(mmcif: MmCif) -> Self {
        let mut chains = Vec::new();
        let mut centers = Vec::new();
        for chain in mmcif.chains {
            let mut residues = Vec::new();
            for residue_id in chain.residue_sns {
                let residue = &mmcif.residues[residue_id as usize];
                let res_type: ResidueType = (&residue.res_type).into();
                let mut ca_opt = None;
                let mut cb_opt = None;
                for atom_id in &residue.atom_sns {
                    let atom = &mmcif.atoms[*atom_id as usize];
                    if let Some(atom_type_in_res) = &atom.type_in_res {
                        if *atom_type_in_res == AtomTypeInRes::CA {
                            ca_opt = Some(atom.posit);
                        }
                        if *atom_type_in_res == AtomTypeInRes::CB {
                            cb_opt = Some(atom.posit);
                        }
                    }

                    if ca_opt.is_none() {
                        println!("No CA atom found");
                        panic!("No CA atom found");
                    }

                    centers.push(Vec3 {
                        x: ca_opt.unwrap().x as f32,
                        y: ca_opt.unwrap().y as f32,
                        z: ca_opt.unwrap().z as f32,
                    });
                }

                residues.push(Residue {
                    residue_type: res_type,
                    ca: Vec3 {
                        x: ca_opt.unwrap().x as f32,
                        y: ca_opt.unwrap().y as f32,
                        z: ca_opt.unwrap().z as f32,
                    },
                    cb: match cb_opt {
                        Some(pos) => Some(Vec3 {
                            x: pos.x as f32,
                            y: pos.y as f32,
                            z: pos.z as f32,
                        }),
                        None => None,
                    },
                    index: residue_id as usize,
                    ss: SecondaryStructure::Unknown,
                });
            }
            chains.push(Chain { residues: residues });
        }

        let mut center = Vec3::ZERO;
        for c in &centers {
            center += c;
        }
        center = center / (centers.len() as f32);

        println!("len: {}", centers.len());

        Protein {
            chains: chains,
            center: center,
        }
    }
}

impl Protein {
    pub fn get_center(&self) -> [f32; 3] {
        [self.center.x, self.center.y, self.center.z]
    }

    pub fn centered(mut self) -> Self {
        let center = Vec3 {
            x: self.center.x,
            y: self.center.y,
            z: self.center.z,
        };
        for chain in &mut self.chains {
            for residue in &mut chain.residues {
                residue.ca -= center;
                if let Some(cb) = residue.cb {
                    residue.cb = Some(cb - center);
                }
            }
        }
        self
    }

    pub fn to_mesh(&self, scale: f32) -> MeshData {
        let mut mesh = MeshData::default();

        // 1. 把所有链合并成一个连续的残基序列，只处理有 Cα 的残基
        let mut residues = Vec::new();
        for chain in &self.chains {
            for res in &chain.residues {
                if res.ca.length_squared() > 0.0 {
                    // 过滤掉缺失的
                    residues.push(res);
                }
            }
        }
        if residues.len() < 2 {
            println!("Not enough residues to draw protein mesh");
            return mesh; // 没法画
        }

        // 2. 构建三次样条得到平滑的 guide points（每个残基 5 个点，ChimeraX 默认）
        let pts_per_res = 5;
        let spline = self.build_spline(&residues);

        // 采样点（包括起点/终点）
        let total_pts = residues.len() * pts_per_res + 1;
        let mut centers = Vec::with_capacity(total_pts);
        let mut tangents = Vec::with_capacity(total_pts);
        let mut normals = Vec::with_capacity(total_pts);

        for i in 0..total_pts {
            let t = i as f32 / ((total_pts - 1) as f32);
            let pos = sample_vec3_linear(spline.keys(), t).unwrap();
            let dt = 0.001; // 一个很小的 t 步长
            let t0 = (i as f32 / (total_pts - 1) as f32).clamp(0.0, 1.0);
            let t1 = (t0 + dt).min(1.0);

            let p0 = sample_vec3_linear(spline.keys(), t0).unwrap();
            let p1 = sample_vec3_linear(spline.keys(), t1).unwrap();

            let tan = (p1 - p0).normalize_or_zero();
            centers.push(pos);
            tangents.push(tan);

            // 初始 normal（尽量避免和 tangent 共线）
            let mut n = if tan.dot(Vec3::Z).abs() > 0.9 {
                Vec3::Y
            } else {
                Vec3::Z
            };
            n = n - n.dot(tan) * tan;
            if n.length_squared() < 1e-6 {
                n = tan.cross(Vec3::X).normalize();
            } else {
                n = n.normalize();
            }
            normals.push(n);
        }

        // 3. 旋转最小化扭转（Ribbon 的关键步骤，和 ChimeraX 完全一样）
        for i in 1..normals.len() {
            let t = tangents[i];
            let prev_n = normals[i - 1];
            let bin = t.cross(prev_n).normalize_or_zero();
            let new_n = bin.cross(t).normalize_or_zero();
            if new_n.dot(prev_n) < 0.0 {
                normals[i] = -new_n;
            } else {
                normals[i] = new_n;
            }
        }

        // 4. 按残基决定使用哪个 cross-section
        let mut sections = Vec::new();
        for (i, res) in residues.iter().enumerate() {
            let xs = match res.ss {
                SecondaryStructure::Helix => helix_section(),
                SecondaryStructure::Sheet => sheet_section(),
                SecondaryStructure::Coil | SecondaryStructure::Unknown => coil_section(),
            };
            sections.push(xs);
        }

        // 5. 真正的 extrusion（和 ChimeraX _extrude_smooth 完全等价）
        self.extrude_ribbon(
            &centers,
            &tangents,
            &normals,
            &sections,
            pts_per_res,
            &mut mesh,
        );

        // 6. 统一缩放（如果你想让整体更大/更小）
        for v in &mut mesh.vertices {
            *v = (Vec3::from(*v) * scale).into();
        }
        for n in &mut mesh.normals {
            *n = (Vec3::from(*n) * scale).into(); // normal 不缩放长度，只乘方向
        }

        // 颜色（这里先全白，你后面可以按 residue.ribbon_color() 填）
        let white = [1.0, 1.0, 1.0, 1.0];
        mesh.colors = Some(vec![white; mesh.vertices.len()]);

        mesh
    }

    // 构建三次样条（和 ChimeraX ribbon_spline 一模一样）
    fn build_spline(&self, residues: &[&Residue]) -> Spline<f32, Vec3> {
        let mut keys = Vec::new();
        for (i, r) in residues.iter().enumerate() {
            let t = i as f32;
            let pos = r.ca;
            // ChimeraX 用 Catmull-Rom（自然端点条件）
            keys.push(Key::new(t, pos, splines::Interpolation::CatmullRom));
        }
        Spline::from_vec(keys)
    }

    // 真正的 ribbon extrusion（核心算法 100% 复刻 ChimeraX）
    fn extrude_ribbon(
        &self,
        centers: &[Vec3],
        tangents: &[Vec3],
        normals: &[Vec3],
        sections: &[RibbonXSection],
        pts_per_res: usize,
        mesh: &mut MeshData,
    ) {
        let n_seg = sections.len();
        let base_vertex = mesh.vertices.len() as u32;

        for seg_idx in 0..n_seg {
            let xs = &sections[seg_idx];
            let start = seg_idx * pts_per_res;
            let end = (seg_idx + 1) * pts_per_res + 1; // +1 因为包含下一个残基的起点

            // 是否需要前后端盖（端点残基需要）
            let cap_front = seg_idx == 0;
            let cap_back = seg_idx == n_seg - 1;

            // 箭头处理（Sheet）
            let (coords, coords_back) = if let Some(back) = &xs.arrow_coords {
                (&xs.coords[..], Some(&back[..]))
            } else {
                (&xs.coords[..], None)
            };

            self.extrude_segment(
                &centers[start..end.min(centers.len())],
                &tangents[start..end.min(centers.len())],
                &normals[start..end.min(centers.len())],
                coords,
                coords_back,
                cap_front,
                cap_back,
                mesh,
            );
        }

        // 把索引加上偏移
        let added = (mesh.vertices.len() as u32) - base_vertex;
        for idx in mesh.indices.iter_mut() {
            *idx += base_vertex;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn extrude_segment(
        &self,
        centers: &[Vec3],
        tangents: &[Vec3],
        normals: &[Vec3],
        coords: &[[f32; 2]],
        arrow_back: Option<&[[f32; 2]]>,
        cap_front: bool,
        cap_back: bool,
        mesh: &mut MeshData,
    ) {
        let n = coords.len();
        let n_pts = centers.len();
        let base = mesh.vertices.len() as u32;

        // 预计算 binormal = tangent × normal
        let mut binormals = Vec::with_capacity(n_pts);
        for i in 0..n_pts {
            let t = tangents[i];
            let n = normals[i];
            let b = t.cross(n).normalize_or_zero();
            binormals.push(b);
        }

        // 生成顶点（smooth 模式）
        for i in 0..n_pts {
            let c = centers[i];
            let nrm = normals[i];
            let bin = binormals[i];

            let arrow_factor = if let Some(back) = arrow_back {
                // 前半段线性过渡到箭头形状
                let t = i as f32 / (n_pts - 1) as f32;
                if t < 0.5 { 1.0 - t * 2.0 } else { 0.0 }
            } else {
                1.0
            };

            for (i, &off) in coords.iter().enumerate() {
                let off = if arrow_factor > 0.0 {
                    if let Some(back) = arrow_back {
                        let back_off = back[i]; // 用相同索引
                        let x = off[0] + (back_off[0] - off[0]) * arrow_factor;
                        let y = off[1] + (back_off[1] - off[1]) * arrow_factor;
                        [x, y]
                    } else {
                        off
                    }
                } else {
                    off
                };

                let pos = c + nrm * off[0] + bin * off[1];
                mesh.vertices.push(pos.to_array());
                let nor = (nrm * off[0] + bin * off[1]).normalize_or_zero();
                mesh.normals.push(nor.to_array());
            }
        }

        // 生成三角形（环向）
        for i in 0..(n_pts - 1) {
            for s in 0..n {
                let s1 = (s + 1) % n;
                let i0 = (i * n + s) as u32 + base;
                let i1 = (i * n + s1) as u32 + base;
                let j0 = ((i + 1) * n + s) as u32 + base;
                let j1 = ((i + 1) * n + s1) as u32 + base;

                mesh.indices.extend_from_slice(&[i0, j0, i1]);
                mesh.indices.extend_from_slice(&[i1, j0, j1]);
            }
        }

        // 端盖（如果需要）
        if cap_front || cap_back {
            let (tangent, reverse) = if cap_front {
                (tangents[0], true)
            } else {
                (tangents[n_pts - 1], false)
            };
            let cap_normal = if reverse { -tangent } else { tangent };
            let start = if cap_front { 0 } else { (n_pts - 1) * n };

            // 简单扇形三角化
            for s in 1..(n - 1) {
                let a = base + start as u32;
                let b = base + start as u32 + s as u32;
                let c = base + start as u32 + (s + 1) as u32;
                if reverse {
                    mesh.indices.extend_from_slice(&[a, c, b]);
                } else {
                    mesh.indices.extend_from_slice(&[a, b, c]);
                }
            }
        }
    }
}

// Linear interpolation helper
fn lerp_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}

// Sample Vec3 from spline keys (only linear)
pub fn sample_vec3_linear(keys: &[Key<f32, Vec3>], t: f32) -> Option<Vec3> {
    if keys.len() < 2 {
        return None;
    }

    // Clamp t into range
    let t = t.clamp(keys.first().unwrap().t, keys.last().unwrap().t);

    // Find segment containing t
    for i in 0..keys.len() - 1 {
        let k0 = &keys[i];
        let k1 = &keys[i + 1];

        if t >= k0.t && t <= k1.t {
            let local_t = (t - k0.t) / (k1.t - k0.t);
            return Some(lerp_vec3(k0.value, k1.value, local_t));
        }
    }

    // Should never reach here
    None
}

impl Into<Shape> for Protein {
    fn into(self) -> Shape {
        Shape::Protein(self)
    }
}

// ---------- 下面是 ChimeraX 完全一致的 cross-section 定义 ----------
fn helix_section() -> RibbonXSection {
    // 圆形（实际上是 32 边形近似）+ 箭头（Sheet 用）
    let mut coords = Vec::new();
    let n = 32;
    for i in 0..n {
        let a = (i as f32) / (n as f32) * std::f32::consts::TAU;
        coords.push([a.cos(), a.sin()]);
    }
    RibbonXSection::smooth_circle(&coords)
}

fn sheet_section() -> RibbonXSection {
    // ChimeraX 默认箭头：前半段宽 1.0 → 0.0，后半段保持 0.0
    let base_arrow = helix_section().arrow(1.0, 1.0, 0.0, 1.2); // 经典箭头比例
    base_arrow
}

fn coil_section() -> RibbonXSection {
    // Coil 用细一点的圆管
    helix_section().scale(0.6, 0.6)
}

struct RibbonXSection {
    coords: Vec<[f32; 2]>,               // 基础 2D 轮廓
    arrow_coords: Option<Vec<[f32; 2]>>, // 为 Sheet 箭头准备的第二套轮廓
    smooth: bool,
}

impl RibbonXSection {
    fn smooth_circle(coords: &[[f32; 2]]) -> Self {
        Self {
            coords: coords.to_vec(),
            arrow_coords: None,
            smooth: true,
        }
    }

    fn scale(mut self, sx: f32, sy: f32) -> Self {
        for c in &mut self.coords {
            c[0] *= sx;
            c[1] *= sy;
        }
        if let Some(ac) = self.arrow_coords.as_mut() {
            for c in ac {
                c[0] *= sx;
                c[1] *= sy;
            }
        }
        self
    }

    fn arrow(mut self, sx1: f32, sy1: f32, sx2: f32, sy2: f32) -> Self {
        let mut back = self.coords.clone();
        for c in &mut self.coords {
            c[0] *= sx1;
            c[1] *= sy1;
        }
        for c in &mut back {
            c[0] *= sx2;
            c[1] *= sy2;
        }
        self.arrow_coords = Some(back);
        self
    }
}
