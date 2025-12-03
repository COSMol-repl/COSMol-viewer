use glam::Vec3;
use na_seq::Element;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    Shape,
    parser::sdf::MoleculeData,
    scene::InstanceGroups,
    shapes::{sphere::Sphere, stick::Stick},
    utils::{
        Interaction, Interpolatable, IntoInstanceGroups, Logger, MeshData, VisualShape, VisualStyle,
    },
};

use std::collections::HashMap;

pub fn my_color(x: &str) -> Vec3 {
    let element = Element::from_letter(x).unwrap();

    // 优先使用自定义颜色
    match element {
        Element::Hydrogen => Vec3::new(1.0, 1.0, 1.0),
        Element::Carbon => Vec3::new(0.3, 0.3, 0.3),
        Element::Nitrogen => Vec3::new(0.2, 0.4, 1.0),
        Element::Oxygen => Vec3::new(1.0, 0.0, 0.0),
        Element::Fluorine => Vec3::new(0.0, 0.8, 0.0),
        Element::Phosphorus => Vec3::new(1.0, 0.5, 0.0),
        Element::Sulfur => Vec3::new(1.0, 1.0, 0.0),
        Element::Chlorine => Vec3::new(0.0, 0.8, 0.0),
        Element::Bromine => Vec3::new(0.6, 0.2, 0.2),
        Element::Iodine => Vec3::new(0.4, 0.0, 0.8),
        _ => element.color().into(), // 其他未定义的元素
    }
}

pub fn my_radius(x: &str) -> f32 {
    let element = Element::from_letter(x).unwrap();
    match element {
        Element::Hydrogen => 1.20,
        _ => element.vdw_radius(), // 默认半径
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BondType {
    SINGLE = 1,
    DOUBLE = 2,
    TRIPLE = 3,
    AROMATIC = 0,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Molecules {
    pub atom_types: Vec<String>,
    pub atoms: Vec<Vec3>,
    pub bond_types: Vec<BondType>,
    pub bonds: Vec<[usize; 2]>,
    pub quality: u32,

    pub style: VisualStyle,
    pub interaction: Interaction,
}

impl Interpolatable for Molecules {
    fn interpolate(&self, other: &Self, t: f32, logger: impl Logger) -> Self {
        // 检查原子数量是否匹配
        if self.atoms.len() != other.atoms.len() {
            logger.error(format!(
                "Interpolation aborted: atom count differs (self: {}, other: {}). \
                Smooth interpolation requires scenes with identical atom structures.",
                self.atoms.len(),
                other.atoms.len()
            ));
            panic!("Smooth interpolation requires matching atom structures.");
        }

        // 检查键数量是否匹配（可选，根据需要）
        if self.bonds.len() != other.bonds.len() {
            logger.error(format!(
                "Interpolation aborted: bond topology differs (self: {}, other: {}). \
                Smooth interpolation cannot proceed with different bonding graphs.",
                self.bonds.len(),
                other.bonds.len()
            ));
            panic!("Smooth interpolation requires matching bond topology.");
        }

        // 原子坐标插值
        let atoms: Vec<Vec3> = self
            .atoms
            .iter()
            .zip(&other.atoms)
            .map(|(a, b)| {
                Vec3::new(
                    a[0] * (1.0 - t) + b[0] * t,
                    a[1] * (1.0 - t) + b[1] * t,
                    a[2] * (1.0 - t) + b[2] * t,
                )
            })
            .collect();

        Self {
            atom_types: self.atom_types.clone(), // 假设 atom 类型不变
            atoms,
            bond_types: self.bond_types.clone(),
            bonds: self.bonds.clone(),
            quality: ((self.quality as f32) * (1.0 - t) + (other.quality as f32) * t) as u32,
            style: self.style.clone(),
            interaction: self.interaction.clone(),
        }
    }
}

impl Into<Shape> for Molecules {
    fn into(self) -> Shape {
        Shape::Molecules(self)
    }
}

impl Molecules {
    pub fn new(molecule_data: MoleculeData) -> Self {
        let mut atom_types = Vec::new();
        let mut atoms = Vec::new();
        let mut bond_set = HashMap::new(); // prevent duplicates
        let mut bond_types = Vec::new();
        let mut bonds = Vec::new();

        for molecule in molecule_data {
            for atom in &molecule {
                // 原子类型
                let atom_type: Element = Element::from_letter(&atom.elem).unwrap();
                atom_types.push(atom_type);

                // 原子坐标
                atoms.push(Vec3::new(atom.x, atom.y, atom.z));
            }

            // 处理键（避免重复）
            for atom in &molecule {
                let from = atom.index;
                for (i, &to) in atom.bonds.iter().enumerate() {
                    let to = to;
                    let key = if from < to { (from, to) } else { (to, from) };

                    if !bond_set.contains_key(&key) {
                        bond_set.insert(key, true);
                        bonds.push([key.0, key.1]);

                        let order = atom.bond_order[i];
                        let bond_type = match order {
                            1 => BondType::SINGLE,
                            2 => BondType::DOUBLE,
                            3 => BondType::TRIPLE,
                            _ => BondType::AROMATIC, // fallback
                        };
                        bond_types.push(bond_type);
                    }
                }
            }
        }

        Self {
            atom_types: atom_types
                .iter()
                .map(|&element| element.to_letter())
                .collect(),
            atoms,
            bond_types,
            bonds,
            quality: 6,
            style: VisualStyle {
                opacity: 1.0,
                visible: true,
                ..Default::default()
            },
            interaction: Default::default(),
        }
    }

    pub fn get_center(&self) -> [f32; 3] {
        if self.atoms.is_empty() {
            return [0.0; 3];
        }

        // 1. 累加所有原子坐标
        let mut center = [0.0f32; 3];
        for pos in &self.atoms {
            center[0] += pos[0];
            center[1] += pos[1];
            center[2] += pos[2];
        }

        // 2. 计算平均值
        let count = self.atoms.len() as f32;
        center[0] /= count;
        center[1] /= count;
        center[2] /= count;

        center
    }

    /// Centers the molecule by translating all atoms so that the geometric center
    /// is at the origin (0.0, 0.0, 0.0).
    pub fn centered(mut self) -> Self {
        let center = self.get_center();
        for atom in &mut self.atoms {
            atom[0] -= center[0];
            atom[1] -= center[1];
            atom[2] -= center[2];
        }

        self
    }

    pub fn reset_color(mut self) -> Self {
        self.style_mut().color = None;
        self
    }

    pub fn to_mesh(&self, _scale: f32) -> MeshData {
        MeshData::default()
        // return MeshData::default();

        // let mut vertices = Vec::new();
        // let mut normals = Vec::new();
        // let mut indices = Vec::new();
        // let mut colors = Vec::new();

        // let mut index_offset = 0;

        // // 1. 原子 -> Sphere
        // for (i, pos) in self.atoms.iter().enumerate() {
        //     let radius = self
        //         .atom_types
        //         .get(i)
        //         .unwrap_or(&AtomType::Unknown)
        //         .radius()
        //         * 0.2;
        //     let color = self
        //         .style
        //         .color
        //         .unwrap_or(self.atom_types.get(i).unwrap_or(&AtomType::Unknown).color());

        //     let mut sphere = Sphere::new(*pos, radius);
        //     sphere.interaction = self.interaction;
        //     sphere = sphere.color(color).opacity(self.style.opacity);

        //     let mesh = sphere.to_mesh(1.0);

        //     // 合并 mesh
        //     for v in mesh.vertices {
        //         vertices.push(v.map(|x| x * scale));
        //     }
        //     for n in mesh.normals {
        //         normals.push(n.map(|x| x * scale));
        //     }
        //     for c in mesh.colors.unwrap() {
        //         colors.push(c);
        //     }
        //     for idx in mesh.indices {
        //         indices.push(idx + index_offset as u32);
        //     }

        //     index_offset = vertices.len() as u32;
        // }

        // // 2. 键 -> Stick
        // for (_i, bond) in self.bonds.iter().enumerate() {
        //     for (_i, bond) in self.bonds.iter().enumerate() {
        //         let [a, b] = *bond;
        //         let pos_a = self.atoms[a as usize];
        //         let pos_b = self.atoms[b as usize];

        //         // 获取原子颜色
        //         let color_a = match self
        //             .atom_types
        //             .get(a as usize)
        //             .unwrap_or(&AtomType::Unknown)
        //         {
        //             AtomType::C => [0.75, 0.75, 0.75],
        //             other => other.color(),
        //         };

        //         let color_b = match self
        //             .atom_types
        //             .get(b as usize)
        //             .unwrap_or(&AtomType::Unknown)
        //         {
        //             AtomType::C => [0.75, 0.75, 0.75],
        //             other => other.color(),
        //         };

        //         // 计算中点
        //         let mid = [
        //             0.5 * (pos_a[0] + pos_b[0]),
        //             0.5 * (pos_a[1] + pos_b[1]),
        //             0.5 * (pos_a[2] + pos_b[2]),
        //         ];

        //         // bond 一：A -> 中点，颜色 A
        //         let stick_a = Stick::new(pos_a, mid, 0.15)
        //             .color(color_a)
        //             .opacity(self.style.opacity);
        //         let mesh_a = stick_a.to_mesh(1.0);
        //         for v in mesh_a.vertices {
        //             vertices.push(v.map(|x| x * scale));
        //         }
        //         for n in mesh_a.normals {
        //             normals.push(n.map(|x| x * scale));
        //         }
        //         for c in mesh_a.colors.unwrap() {
        //             colors.push(c);
        //         }
        //         for idx in mesh_a.indices {
        //             indices.push(idx + index_offset as u32);
        //         }
        //         index_offset = vertices.len() as u32;

        //         // bond 二：B -> 中点，颜色 B
        //         let stick_b = Stick::new(pos_b, mid, 0.15)
        //             .color(color_b)
        //             .opacity(self.style.opacity);
        //         let mesh_b = stick_b.to_mesh(1.0);
        //         for v in mesh_b.vertices {
        //             vertices.push(v.map(|x| x * scale));
        //         }
        //         for n in mesh_b.normals {
        //             normals.push(n.map(|x| x * scale));
        //         }
        //         for c in mesh_b.colors.unwrap() {
        //             colors.push(c);
        //         }
        //         for idx in mesh_b.indices {
        //             indices.push(idx + index_offset as u32);
        //         }
        //         index_offset = vertices.len() as u32;
        //     }
        // }

        // MeshData {
        //     vertices,
        //     normals,
        //     indices,
        //     colors: Some(colors),
        //     transform: None,
        //     is_wireframe: self.style.wireframe,
        // }
    }
}

impl IntoInstanceGroups for Molecules {
    fn to_instance_group(&self, scale: f32) -> InstanceGroups {
        let mut groups = InstanceGroups::default();

        for (i, pos) in self.atoms.iter().enumerate() {
            let sphere_instance = Sphere::new(
                pos.to_array(),
                self.atom_types.get(i).map(|x| my_radius(x) * 0.2).unwrap(),
            )
            .color(
                self.style
                    .color
                    .unwrap_or(self.atom_types.get(i).map(|x| my_color(x)).unwrap())
                    .into(),
            )
            .opacity(self.style.opacity);

            groups.spheres.push(sphere_instance.to_instance(scale));
        }

        for (i, bond) in self.bonds.iter().enumerate() {
            let [a, b] = bond;
            let pos_a = self.atoms[*a];
            let pos_b = self.atoms[*b];

            let bond_type = self.bond_types.get(i).unwrap_or(&BondType::SINGLE);

            // 方向向量
            let dir = [
                pos_b[0] - pos_a[0],
                pos_b[1] - pos_a[1],
                pos_b[2] - pos_a[2],
            ];

            // 归一化方向
            let norm = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            let dir_n = [dir[0] / norm, dir[1] / norm, dir[2] / norm];

            // === Step 1: 先找 A 的邻居方向（排除 B）===
            let mut neighbor_dir_opt = None;
            for (_j, other_bond) in self.bonds.iter().enumerate() {
                let [x, y] = other_bond;
                if x == a && y != b {
                    let pos_n = self.atoms[*y];
                    neighbor_dir_opt = Some([
                        pos_n[0] - pos_a[0],
                        pos_n[1] - pos_a[1],
                        pos_n[2] - pos_a[2],
                    ]);
                    break;
                } else if y == a && x != b {
                    let pos_n = self.atoms[*x];
                    neighbor_dir_opt = Some([
                        pos_n[0] - pos_a[0],
                        pos_n[1] - pos_a[1],
                        pos_n[2] - pos_a[2],
                    ]);
                    break;
                }
            }

            // ✅ 若 A 没有邻居，则去找 B 的邻居
            if neighbor_dir_opt.is_none() {
                for (_j, other_bond) in self.bonds.iter().enumerate() {
                    let [x, y] = other_bond;
                    if x == b && y != a {
                        let pos_n = self.atoms[*y];
                        neighbor_dir_opt = Some([
                            pos_n[0] - pos_b[0],
                            pos_n[1] - pos_b[1],
                            pos_n[2] - pos_b[2],
                        ]);
                        break;
                    } else if y == b && x != a {
                        let pos_n = self.atoms[*x];
                        neighbor_dir_opt = Some([
                            pos_n[0] - pos_b[0],
                            pos_n[1] - pos_b[1],
                            pos_n[2] - pos_b[2],
                        ]);
                        break;
                    }
                }
            }

            // === Step 2: 计算 offset 方向 ===
            let offset = if let Some(nd) = neighbor_dir_opt {
                // 用邻居方向构造共面偏移
                let nd_norm = (nd[0] * nd[0] + nd[1] * nd[1] + nd[2] * nd[2]).sqrt();
                let nd_n = [nd[0] / nd_norm, nd[1] / nd_norm, nd[2] / nd_norm];

                // 计算 nd_n 在 dir_n 方向的投影分量
                let dot = nd_n[0] * dir_n[0] + nd_n[1] * dir_n[1] + nd_n[2] * dir_n[2];
                let proj = [dot * dir_n[0], dot * dir_n[1], dot * dir_n[2]];

                // 去掉投影分量，得到“共面但不沿键方向”的偏移矢量
                [nd_n[0] - proj[0], nd_n[1] - proj[1], nd_n[2] - proj[2]]
            } else {
                // ✅ A 和 B 都没有邻居 → 回到默认垂直方向
                let up = if dir_n[0].abs() < 0.9 {
                    [1.0, 0.0, 0.0]
                } else {
                    [0.0, 1.0, 0.0]
                };
                [
                    dir_n[1] * up[2] - dir_n[2] * up[1],
                    dir_n[2] * up[0] - dir_n[0] * up[2],
                    dir_n[0] * up[1] - dir_n[1] * up[0],
                ]
            };

            // 归一化 offset
            let off_norm =
                (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
            let off_n = [
                offset[0] / off_norm,
                offset[1] / off_norm,
                offset[2] / off_norm,
            ];

            // 偏移距离（可调）
            let d = 0.22;

            // 颜色和半径与原来一致
            let color_a = self.style.color.unwrap_or(
                self.atom_types
                    .get(*a)
                    .map(|x| match x.as_str() {
                        "C" => Vec3::new(0.75, 0.75, 0.75),
                        _ => my_color(x),
                    })
                    .unwrap(),
            );
            let color_b = self.style.color.unwrap_or(
                self.atom_types
                    .get(*b)
                    .map(|x| match x.as_str() {
                        "C" => Vec3::new(0.75, 0.75, 0.75),
                        _ => my_color(x),
                    })
                    .unwrap(),
            );

            // 根据键类型生成多个 stick
            let (num_sticks, radius) = match bond_type {
                BondType::SINGLE => (1, 0.135),
                BondType::DOUBLE => (2, 0.09),
                BondType::TRIPLE => (3, 0.05),
                _ => (1, 0.15), // aromatic等以后再处理
            };

            for k in 0..num_sticks {
                let offset_mul = (k as f32 - (num_sticks - 1) as f32 * 0.5) * d;

                let pos_a_k = [
                    pos_a[0] + off_n[0] * offset_mul,
                    pos_a[1] + off_n[1] * offset_mul,
                    pos_a[2] + off_n[2] * offset_mul,
                ];
                let pos_b_k = [
                    pos_b[0] + off_n[0] * offset_mul,
                    pos_b[1] + off_n[1] * offset_mul,
                    pos_b[2] + off_n[2] * offset_mul,
                ];

                // A -> 中点
                let stick_a = Stick::new(
                    pos_a_k,
                    [
                        0.5 * (pos_a_k[0] + pos_b_k[0]),
                        0.5 * (pos_a_k[1] + pos_b_k[1]),
                        0.5 * (pos_a_k[2] + pos_b_k[2]),
                    ],
                    radius,
                )
                .color(color_a.into())
                .opacity(self.style.opacity);

                groups.sticks.push(stick_a.to_instance(scale));

                // B -> 中点
                let stick_b = Stick::new(
                    pos_b_k,
                    [
                        0.5 * (pos_a_k[0] + pos_b_k[0]),
                        0.5 * (pos_a_k[1] + pos_b_k[1]),
                        0.5 * (pos_a_k[2] + pos_b_k[2]),
                    ],
                    radius,
                )
                .color(color_b.into())
                .opacity(self.style.opacity);

                groups.sticks.push(stick_b.to_instance(scale));
            }
        }
        groups
    }
}

impl VisualShape for Molecules {
    fn style_mut(&mut self) -> &mut VisualStyle {
        &mut self.style
    }
}
