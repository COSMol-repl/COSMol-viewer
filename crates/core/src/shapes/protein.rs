use crate::Shape;
use crate::parser::CompSS::SecondaryStructureCalculator;
use crate::parser::mmcif::Chain;
use crate::parser::mmcif::MmCif;
use crate::parser::mmcif::Residue;
use crate::parser::mmcif::ResidueType;
use crate::parser::mmcif::SecondaryStructure;
use crate::shapes::protein::ResidueType::AminoAcid;
use crate::utils::{MeshData, VisualShape, VisualStyle};
use glam::{Mat3, Quat, Vec3, Vec4};
use na_seq::AtomTypeInRes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Protein {
    pub chains: Vec<Chain>,
    pub center: Vec3,
}

fn torsion(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> f32 {
    let b1 = b - a;
    let b2 = c - b;
    let b3 = d - c;

    let n1 = b1.cross(b2);
    let n2 = b2.cross(b3);

    let y = b2.normalize().dot(n1.cross(n2));
    let x = n1.dot(n2);

    y.atan2(x) // 返回弧度
}

struct TmpBackbone {
    ca: Vec3,
    c: Vec3,
    n: Vec3,
}

impl Protein {
    pub fn new(mmcif: MmCif) -> Self {
        let mut chains = Vec::new();
        let mut centers = Vec::new();
        let mut residue_index = 0;

        for chain in mmcif.chains {
            let mut residues = Vec::new();

            for residue_sns in chain.residue_sns {
                let residue = &mmcif.residues[residue_index];
                residue_index += 1;
                let amino_acid = match residue.res_type.clone() {
                    AminoAcid(aa) => aa,
                    _ => continue,
                };
                let mut ca_opt = None;
                let mut c_opt = None;
                let mut n_opt = None;
                let mut o_opt = None;
                for atom_sn in &residue.atom_sns {
                    let atom = &mmcif.atoms[*atom_sn as usize - 1];
                    if let Some(atom_type_in_res) = &atom.type_in_res {
                        if *atom_type_in_res == AtomTypeInRes::C {
                            c_opt = Some(atom.posit);
                        }
                        if *atom_type_in_res == AtomTypeInRes::N {
                            n_opt = Some(atom.posit);
                        }
                        if *atom_type_in_res == AtomTypeInRes::CA {
                            ca_opt = Some(atom.posit);
                        }
                        if *atom_type_in_res == AtomTypeInRes::O {
                            o_opt = Some(atom.posit);
                        }
                    }
                }

                if ca_opt.is_none() {
                    println!(
                        "No CA atom found for chain {} residue {}",
                        chain.id, residue_sns
                    );
                    continue;
                }
                if c_opt.is_none() {
                    println!(
                        "No C atom found for chain {} residue {}",
                        chain.id, residue_sns
                    );
                    continue;
                }
                if n_opt.is_none() {
                    println!(
                        "No N atom found for chain {} residue {}",
                        chain.id, residue_sns
                    );
                    continue;
                }
                if o_opt.is_none() {
                    println!(
                        "No O atom found for chain {} residue {}",
                        chain.id, residue_sns
                    );
                    continue;
                }

                let (ca, c, n, o) = (
                    ca_opt.unwrap(),
                    c_opt.unwrap(),
                    n_opt.unwrap(),
                    o_opt.unwrap(),
                );

                centers.push(Vec3::new(ca.x as f32, ca.y as f32, ca.z as f32));

                residues.push(Residue {
                    residue_type: amino_acid,
                    ca: ca,
                    c: c,
                    n: n,
                    o: o,
                    h: None,
                    sns: residue_sns as usize,
                    ss: None,
                });
            }

            chains.push(Chain {
                residues: residues,
                id: chain.id,
            });
        }

        let mut center = Vec3::ZERO;
        for c in &centers {
            center += c;
        }
        center = center / (centers.len() as f32);

        Protein {
            chains: chains,
            center: center,
        }
    }
}

impl VisualShape for Protein {
    fn style_mut(&mut self) -> &mut VisualStyle {
        unimplemented!()
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
                residue.c -= center;
                residue.n -= center;
                residue.o -= center;
                residue.o -= center;
                if let Some(h) = residue.h {
                    residue.h = Some(h - center);
                }
            }
        }
        self.center = Vec3::ZERO;
        self
    }

    fn catmull_rom_chain(&self, positions: &[Vec3]) -> Vec<Vec3> {
        let n = positions.len();
        if n < 2 {
            return positions.to_vec();
        }

        let mut path = Vec::with_capacity(n * 5 + 1);
        path.push(positions[0]); // 第一个点

        for i in 0..n - 1 {
            let p0 = if i > 0 {
                positions[i - 1]
            } else {
                positions[0]
            };
            let p1 = positions[i];
            let p2 = positions[i + 1];
            let p3 = if i + 2 < n {
                positions[i + 2]
            } else {
                positions[i + 1]
            };

            // 生成 5 个中间点 + 1终点（下一个起点）
            for j in 1..=5 {
                let t = j as f32 / 5.0;
                path.push(catmull_rom(p0, p1, p2, p3, t));
            }
        }

        path
    }

    pub fn to_mesh(&self, scale: f32) -> MeshData {
        let mut final_mesh = MeshData::default();
        for chain in &self.chains {
            let mut mesh = MeshData::default();

            let residues: Vec<&Residue> = chain
                .residues
                .iter()
                .filter(|r| r.ca.length_squared() > 1e-6)
                .collect();

            let ca_positions: Vec<Vec3> = residues.iter().map(|r| r.ca).collect();

            if ca_positions.len() < 2 {
                println!("chain {} has less than 2 residues, skipping", chain.id);
                continue;
            }

            // 直接生成平滑路径 + 切线（不再依赖 splines）
            let path = self.catmull_rom_chain(&ca_positions);

            // 把这段完整替换你原来的 centers/tangents/normals 计算部分
            let mut centers = Vec::with_capacity(path.len());
            let mut tangents = Vec::with_capacity(path.len());
            let mut normals = Vec::with_capacity(path.len());
            // let mut ss = Vec::with_capacity(path.len());

            let n = path.len();

            // 1. 填充 centers 和 tangents（你原来的完全正确）
            for i in 0..n {
                let pos = path[i];
                centers.push(pos);

                let tan = if i == 0 {
                    (path[1] - path[0]).normalize()
                } else if i == n - 1 {
                    (path[n - 1] - path[n - 2]).normalize()
                } else {
                    (path[i + 1] - path[i - 1]).normalize()
                };
                tangents.push(tan);
            }

            // 2. 统一的初始法线函数
            fn initial_normal(t: Vec3) -> Vec3 {
                // 优先选 Z，避免和切线几乎平行
                if t.dot(Vec3::Z).abs() < 0.98 {
                    t.cross(Vec3::Z).normalize()
                } else {
                    t.cross(Vec3::X).normalize()
                }
            }

            // === 普通情况：Bishop Frame / Parallel Transport Frame（最小扭转）===
            let mut current_normal = initial_normal(tangents[0]);
            normals.push(current_normal);

            for i in 1..centers.len() {
                let prev_t = tangents[i - 1];
                let curr_t = tangents[i];

                // 平行传输
                let rotation_axis = prev_t.cross(curr_t);
                if rotation_axis.length_squared() > 1e-6 {
                    let rotation_angle = prev_t.angle_between(curr_t);
                    let rotation = Quat::from_axis_angle(rotation_axis.normalize(), rotation_angle);
                    current_normal = rotation * current_normal;
                }
                normals.push(current_normal);
            }

            // sections 和之前一样
            let sections: Vec<RibbonXSection> = chain
                .get_ss()
                .iter()
                .map(|r| match r {
                    SecondaryStructure::Helix => helix_section(),
                    SecondaryStructure::Sheet => sheet_section(),
                    _ => coil_section(),
                })
                .collect();

            // extrusion（保持你最新的 extrude_ribbon_corrected）
            self.extrude_ribbon_corrected(&centers, &tangents, &normals, &sections, 5, &mut mesh);

            // 缩放 + 颜色
            for v in &mut mesh.vertices {
                *v = *v * scale;
            }
            mesh.colors = Some(vec![Vec4::new(1.0, 1.0, 1.0, 1.0); mesh.vertices.len()]);

            final_mesh.append(&mesh);
        }
        final_mesh
    }

    // 完全修正版的 extrusion（不再有任何越界、箭头方向、端盖问题）
    fn extrude_ribbon_corrected(
        &self,
        centers: &[Vec3],
        tangents: &[Vec3],
        normals: &[Vec3],
        sections: &[RibbonXSection],
        pts_per_res: usize,
        mesh: &mut MeshData,
    ) {
        let base_v = mesh.vertices.len() as u32;

        for (seg, xs) in sections.iter().enumerate() {
            let start = seg * pts_per_res;
            // 注意：最后一个 segment 多取 pts_per_res 个点（共享下一个 segment 的起点）
            let end = if seg + 1 < sections.len() {
                (seg + 1) * pts_per_res + 1
            } else {
                centers.len() // 最后一个正好到结尾
            };

            let cap_front = seg == 0;
            let cap_back = seg + 1 == sections.len();

            let (coords, arrow_back) = if xs.arrow_coords.is_some() {
                (&xs.coords[..], xs.arrow_coords.as_deref())
            } else {
                (&xs.coords[..], None)
            };

            self.extrude_one_segment(
                &centers[start..end],
                &tangents[start..end],
                &normals[start..end],
                coords,
                arrow_back,
                cap_front,
                cap_back,
                mesh,
                xs.ss,
            );
        }

        // 统一偏移索引
        for idx in &mut mesh.indices {
            *idx += base_v;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn extrude_one_segment(
        &self,
        centers: &[Vec3],
        tangents: &[Vec3],
        normals: &[Vec3],
        coords: &[[f32; 2]],
        arrow_back: Option<&[[f32; 2]]>,
        cap_front: bool,
        cap_back: bool,
        mesh: &mut MeshData,
        ss: SecondaryStructure,
    ) {
        let n_ring = coords.len();
        let n_pts = centers.len();
        let base = mesh.vertices.len() as u32;

        // binormal = T × N
        let binormals: Vec<Vec3> = tangents
            .iter()
            .zip(normals)
            .map(|(t, n)| t.cross(*n).normalize_or_zero())
            .collect();

        // 生成顶点
        for i in 0..n_pts {
            let c = centers[i];
            let n = normals[i];
            let b = binormals[i];

            // Sheet 箭头只在前半段渐变（ChimeraX 就是这么做的）
            for i in 0..n_pts {
                let c = centers[i];
                let n = normals[i];
                let b = binormals[i];

                // === 正确的箭头渐变：只在后半段出现，且尖端在末端 ===
                let arrow_progress = if arrow_back.is_some() {
                    let local_t = i as f32 / (n_pts.saturating_sub(1)) as f32;
                    // 前半段不变，后半段从 0 → 1
                    ((local_t - 0.5) * 2.0).max(0.0).min(1.0)
                } else {
                    0.0
                };

                for ring_idx in 0..n_ring {
                    let mut off = coords[ring_idx];

                    // 只有在 sheet 且 arrow_back 存在时才变形
                    if arrow_progress > 0.0 && arrow_back.is_some() {
                        let back = arrow_back.unwrap()[ring_idx];
                        // 正确线性插值：正常 → 箭头尖
                        let t = arrow_progress;
                        off[0] = off[0] * (1.0 - t) + back[0] * t;
                        off[1] = off[1] * (1.0 - t) + back[1] * t;
                    }

                    let pos = c + n * off[0] + b * off[1];
                    let nor = match ss {
                        SecondaryStructure::Helix => ellipse_normal(n, b, off, 1.0, 0.25),
                        SecondaryStructure::Sheet => match ring_idx {
                            0 | 1 => b,  // 上表面两个点（右上、左上）
                            2 | 3 => -n, // 左侧边两个点（左上、左下） → 朝向 -N（向后）
                            4 | 5 => -b, // 下表面两个点（左下、右下）
                            6 | 7 => n,  // 右侧边两个点（右下、右上） → 朝向 +B（向前）
                            _ => n,
                        }
                        .normalize(),
                        _ => (n * off[0] + b * off[1]).normalize_or_zero(),
                    };
                    mesh.vertices.push(pos);
                    mesh.normals.push(nor);
                }
            }
        }

        // 四边形 → 三角形
        for i in 0..n_pts - 1 {
            for r in 0..n_ring {
                let r_next = (r + 1) % n_ring;

                let i0 = i * n_ring + r;
                let i1 = i * n_ring + r_next;
                let j0 = (i + 1) * n_ring + r;
                let j1 = (i + 1) * n_ring + r_next;

                let a = base + i0 as u32;
                let b = base + i1 as u32;
                let c = base + j1 as u32;
                let d = base + j0 as u32;

                mesh.indices.extend_from_slice(&[a, b, d]);
                mesh.indices.extend_from_slice(&[b, c, d]);
            }
        }

        // 端盖（扇形三角化）
        let mut cap = |start_pt: usize, tangent: Vec3, outward: bool| {
            let center = base + (start_pt * n_ring) as u32;
            let normal = if outward { tangent } else { -tangent };

            // 简单星形三角化（足够光滑）
            for r in 1..n_ring - 1 {
                let a = center;
                let b = center + r as u32;
                let c = center + (r + 1) as u32;

                let v_ab: Vec3 = mesh.vertices[b as usize] - mesh.vertices[a as usize];
                let v_ac: Vec3 = mesh.vertices[c as usize] - mesh.vertices[a as usize];

                if normal.dot(v_ab.cross(v_ac)) > 0.0 {
                    mesh.indices.extend_from_slice(&[a, c, b]);
                } else {
                    mesh.indices.extend_from_slice(&[a, b, c]);
                }
            }
        };

        if cap_front {
            cap(0, tangents[0], false);
        }
        if cap_back {
            cap(n_pts - 1, tangents[n_pts - 1], true);
        }
    }
}

fn ellipse_normal(n: Vec3, b: Vec3, off: [f32; 2], width: f32, height: f32) -> Vec3 {
    let x = off[0];
    let y = off[1];
    let nx = x / (width * width);
    let ny = y / (height * height);
    let nor = n * nx + b * ny;
    nor.normalize_or_zero()
}

// 标准 Catmull-Rom 公式（ChimeraX、Mol*、PyMOL、VMD 全都用这个）
#[inline(always)]
fn catmull_rom(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;

    // Catmull-Rom 系数（tension = 0.5）
    let c0 = -0.5 * t3 + t2 - 0.5 * t;
    let c1 = 1.5 * t3 - 2.5 * t2 + 1.0;
    let c2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
    let c3 = 0.5 * t3 - 0.5 * t2;

    p0 * c0 + p1 * c1 + p2 * c2 + p3 * c3
}

impl Into<Shape> for Protein {
    fn into(self) -> Shape {
        Shape::Protein(self)
    }
}

// ---------- 下面是 ChimeraX 完全一致的 cross-section 定义 ----------
fn helix_section() -> RibbonXSection {
    // RibbonXSection::smooth_circle().scale(0.25, 1.0)
    RibbonXSection::smooth_circle().scale(1.0, 0.25)
}

fn sheet_section() -> RibbonXSection {
    // ChimeraX 默认箭头：前半段宽 1.0 → 0.0，后半段保持 0.0
    let base_arrow = helix_section().arrow(1.0, 1.0, 0.0, 1.2); // 经典箭头比例
    base_arrow
}

fn coil_section() -> RibbonXSection {
    // Coil 用细一点的圆管
    RibbonXSection::smooth_circle().scale(0.2, 0.2)
}

struct RibbonXSection {
    coords: Vec<[f32; 2]>,               // 基础 2D 轮廓
    arrow_coords: Option<Vec<[f32; 2]>>, // 为 Sheet 箭头准备的第二套轮廓
    ss: SecondaryStructure,
    _smooth: bool,
}

impl RibbonXSection {
    fn smooth_circle() -> Self {
        let mut coords = Vec::new();
        let n = 32;
        for i in 0..n {
            let a = (i as f32) / (n as f32) * std::f32::consts::TAU;
            coords.push([a.cos(), a.sin()]);
        }
        Self {
            coords: coords.to_vec(),
            arrow_coords: None,
            ss: SecondaryStructure::Coil,
            _smooth: true,
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
        self.ss = SecondaryStructure::Helix;
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
        Self {
            coords: [
                [0.2, 1.0],
                [-0.2, 1.0],
                [-0.2, 1.0],
                [-0.2, -1.0],
                [-0.2, -1.0],
                [0.2, -1.0],
                [0.2, -1.0],
                [0.2, 1.0],
            ]
            .into(),
            arrow_coords: None,
            ss: SecondaryStructure::Sheet,
            _smooth: true,
        }
    }
}
