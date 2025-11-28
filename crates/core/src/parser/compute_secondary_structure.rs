use crate::parser::mmcif::Residue;
use crate::parser::mmcif::SecondaryStructure;
use glam::Vec3;
use na_seq::AminoAcid;

// 氢键类型
#[derive(Debug, Clone, Copy)]
pub struct HydrogenBond {
    pub donor_idx: usize,
    pub acceptor_idx: usize,
    pub energy: f32,
}

// 二级结构计算器
pub struct SecondaryStructureCalculator {
    pub hbond_cutoff: f32,        // 氢键能量阈值
    pub min_helix_length: usize,  // 最小螺旋长度
    pub min_strand_length: usize, // 最小β链长度
}

impl Default for SecondaryStructureCalculator {
    fn default() -> Self {
        Self {
            hbond_cutoff: -0.5,   // 默认氢键能量阈值
            min_helix_length: 3,  // 默认最小螺旋长度
            min_strand_length: 2, // 默认最小β链长度
        }
    }
}

impl SecondaryStructureCalculator {
    const DSSP_3DONOR: u32 = 0x0001;
    const DSSP_3ACCEPTOR: u32 = 0x0002;
    const DSSP_3HELIX: u32 = 0x0008;

    const DSSP_4DONOR: u32 = 0x0010;
    const DSSP_4ACCEPTOR: u32 = 0x0020;
    const DSSP_4HELIX: u32 = 0x0080;

    const DSSP_5DONOR: u32 = 0x0100;
    const DSSP_5ACCEPTOR: u32 = 0x0200;
    const DSSP_5HELIX: u32 = 0x0800;

    pub fn new() -> Self {
        Self::default()
    }

    /// 计算一条链的二级结构
    pub fn compute_secondary_structure(&self, residues: &[Residue]) -> Vec<SecondaryStructure> {
        if residues.len() < 2 {
            return vec![SecondaryStructure::Coil; residues.len()];
        }

        let residues_with_h = self.add_imide_hydrogens(residues);
        let hbonds = self.find_hydrogen_bonds(&residues_with_h);
        let (helices, strands) = self.identify_secondary_structure(&residues_with_h, &hbonds);

        self.assign_ss_labels(residues.len(), &helices, &strands)
    }

    /// 补全缺失的亚氨基氢原子
    fn add_imide_hydrogens(&self, residues: &[Residue]) -> Vec<Residue> {
        let mut result = residues.to_vec();

        for i in 1..result.len() {
            if result[i].h.is_some() {
                continue;
            }

            if let Some(prev_res) = result.get(i - 1) {
                if let Some(h_coord) = self.calculate_imide_hydrogen(&result[i], prev_res) {
                    result[i].h = Some(h_coord);
                }
            }
        }

        result
    }
    /// 计算亚氨基氢原子位置
    fn calculate_imide_hydrogen(&self, current: &Residue, previous: &Residue) -> Option<Vec3> {
        let n_coord = current.n;
        let ca_coord = current.ca;
        let c_coord = previous.c;
        let o_coord = previous.o;

        // 计算向量
        let n_to_ca = ca_coord - &n_coord;
        let n_to_c = c_coord - &n_coord;
        let c_to_o = o_coord - &c_coord;

        let _ = n_to_ca.normalize();
        let _ = n_to_c.normalize();
        let _ = c_to_o.normalize();

        // 计算角平分线
        let cac_bisect = Vec3 {
            x: n_to_ca.x + n_to_c.x,
            y: n_to_ca.y + n_to_c.y,
            z: n_to_ca.z + n_to_c.z,
        };
        let _ = cac_bisect.normalize();

        // 计算氢原子方向
        let h_direction = Vec3 {
            x: cac_bisect.x + c_to_o.x,
            y: cac_bisect.y + c_to_o.y,
            z: cac_bisect.z + c_to_o.z,
        };
        let _ = h_direction.normalize();

        // 氢键长度约1.01Å
        let nh_length = 1.01;
        Some(Vec3 {
            x: n_coord.x - h_direction.x * nh_length,
            y: n_coord.y - h_direction.y * nh_length,
            z: n_coord.z - h_direction.z * nh_length,
        })
    }

    /// 查找氢键
    fn find_hydrogen_bonds(&self, residues: &[Residue]) -> Vec<HydrogenBond> {
        let mut hbonds = Vec::new();
        let q1 = 0.42;
        let q2 = 0.20;
        let f = 332.0;

        for i in 0..residues.len() {
            let res_i = &residues[i];

            for j in (i + 2)..residues.len() {
                let res_j = &residues[j];

                // 跳过腈氨酸作为给体
                if res_j.residue_type == AminoAcid::Pro || res_j.h.is_none() {
                    continue;
                }

                if let Some(h_coord) = res_j.h {
                    let r_cn = res_i.c.distance(res_j.n);
                    if r_cn > 7.0 {
                        continue; // 优化：距离太远直接跳过
                    }

                    let r_on = res_i.o.distance(res_j.n);
                    let r_ch = res_i.c.distance(h_coord);
                    let r_oh = res_i.o.distance(h_coord);

                    // 计算氢键能量
                    let energy = q1 * q2 * (1.0 / r_on + 1.0 / r_ch - 1.0 / r_oh - 1.0 / r_cn) * f;

                    if energy < self.hbond_cutoff {
                        hbonds.push(HydrogenBond {
                            donor_idx: j,
                            acceptor_idx: i,
                            energy,
                        });
                    }
                }

                // 检查反向氢键
                if res_i.residue_type == AminoAcid::Pro || res_i.h.is_none() {
                    continue;
                }

                if let Some(h_coord) = res_i.h {
                    let r_cn = res_j.c.distance(res_i.n);
                    if r_cn > 7.0 {
                        continue;
                    }

                    let r_on = res_j.o.distance(res_i.n);
                    let r_ch = res_j.c.distance(h_coord);
                    let r_oh = res_j.o.distance(h_coord);

                    let energy = q1 * q2 * (1.0 / r_on + 1.0 / r_ch - 1.0 / r_oh - 1.0 / r_cn) * f;

                    if energy < self.hbond_cutoff {
                        hbonds.push(HydrogenBond {
                            donor_idx: i,
                            acceptor_idx: j,
                            energy,
                        });
                    }
                }
            }
        }

        hbonds
    }

    /// 识别二级结构
    fn identify_secondary_structure(
        &self,
        residues: &[Residue],
        hbonds: &[HydrogenBond],
    ) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
        let helices = self.find_helices(residues, hbonds);
        let strands = self.find_strands(residues, hbonds);

        (helices, strands)
    }

    /// 查找螺旋
    /// 重新实现：正确的螺旋识别（KSDSSP 逻辑）
    fn find_helices(&self, residues: &[Residue], hbonds: &[HydrogenBond]) -> Vec<(usize, usize)> {
        let n_res = residues.len();
        if n_res < 4 {
            return vec![];
        }

        let mut flags = vec![0u32; n_res];

        // Step 1: 查找所有 3-turn, 4-turn, 5-turn
        self.find_turns(3, &mut flags, hbonds, n_res);
        self.find_turns(4, &mut flags, hbonds, n_res);
        self.find_turns(5, &mut flags, hbonds, n_res);

        // Step 2: 根据连续 acceptor 标记螺旋区域
        self.mark_helices(3, &mut flags, n_res);
        self.mark_helices(4, &mut flags, n_res);
        self.mark_helices(5, &mut flags, n_res);

        // Step 3: 合并连续的螺旋区域（带优先级和边界处理）
        self.collect_helix_regions(&flags, n_res)
    }

    /// 查找 n-turn：i → i+n 的氢键
    fn find_turns(&self, n: usize, flags: &mut [u32], hbonds: &[HydrogenBond], n_res: usize) {
        let max_i = n_res.saturating_sub(n);
        for i in 0..max_i {
            let i_n = i + n;

            // 正向：i 是 donor，i+n 是 acceptor
            let forward = hbonds
                .iter()
                .any(|hb| hb.donor_idx == i && hb.acceptor_idx == i_n);
            // 反向：i+n 是 donor，i 是 acceptor  ← 这才是 α-螺旋的主流方向！
            let backward = hbonds
                .iter()
                .any(|hb| hb.donor_idx == i_n && hb.acceptor_idx == i);

            if forward || backward {
                // 不管正向还是反向，只要有氢键就算
                flags[i] |= match n {
                    3 => Self::DSSP_3ACCEPTOR,
                    4 => Self::DSSP_4ACCEPTOR,
                    5 => Self::DSSP_5ACCEPTOR,
                    _ => 0,
                };
                flags[i_n] |= match n {
                    3 => Self::DSSP_3DONOR,
                    4 => Self::DSSP_4DONOR,
                    5 => Self::DSSP_5DONOR,
                    _ => 0,
                };
            }
        }
    }

    /// 连续两个 acceptor → 标记中间为 helix
    fn mark_helices(&self, n: usize, flags: &mut [u32], n_res: usize) {
        let acceptor = match n {
            3 => Self::DSSP_3ACCEPTOR,
            4 => Self::DSSP_4ACCEPTOR,
            5 => Self::DSSP_5ACCEPTOR,
            _ => return,
        };
        let helix_flag = match n {
            3 => Self::DSSP_3HELIX,
            4 => Self::DSSP_4HELIX,
            5 => Self::DSSP_5HELIX,
            _ => return,
        };

        let max_i = n_res.saturating_sub(n);
        for i in 1..max_i {
            if (flags[i - 1] & acceptor) != 0 && (flags[i] & acceptor) != 0 {
                for j in 0..n {
                    if i + j < n_res {
                        flags[i + j] |= helix_flag;
                    }
                }
            }
        }
    }

    /// 合并螺旋区域：优先级 4 > 5 > 3，允许单残基中断，处理边界
    fn collect_helix_regions(&self, flags: &[u32], n_res: usize) -> Vec<(usize, usize)> {
        let mut helices = vec![];
        let mut start = None;
        let mut cur_type = 0; // 3,4,5

        for i in 0..n_res {
            let f = flags[i];

            let is_3 = (f & Self::DSSP_3HELIX) != 0;
            let is_4 = (f & Self::DSSP_4HELIX) != 0;
            let is_5 = (f & Self::DSSP_5HELIX) != 0;

            let helix_type = if is_4 {
                4
            } else if is_5 {
                5
            } else if is_3 {
                3
            } else {
                0
            };

            if helix_type > 0 {
                if start.is_none() {
                    start = Some(i);
                    cur_type = helix_type;
                } else if helix_type != cur_type {
                    // 类型变化，结束前一个
                    if let Some(s) = start {
                        if i - s >= self.min_helix_length {
                            helices.push((s, i - 1));
                        }
                    }
                    start = Some(i);
                    cur_type = helix_type;
                }
                // 否则继续
            } else if let Some(s) = start {
                // 结束螺旋
                if i - s >= self.min_helix_length {
                    helices.push((s, i - 1));
                }
                start = None;
                cur_type = 0;
            }
        }

        // 处理结尾
        if let Some(s) = start {
            if n_res - s >= self.min_helix_length {
                helices.push((s, n_res - 1));
            }
        }

        helices
    }

    /// 查找β链
    fn find_strands(&self, residues: &[Residue], hbonds: &[HydrogenBond]) -> Vec<(usize, usize)> {
        let mut strand_regions = Vec::new();
        let bridges = self.find_beta_bridges(residues, hbonds);
        let ladders = self.build_beta_ladders(&bridges, residues.len());

        for ladder in ladders {
            let start = ladder.0.0.min(ladder.1.0);
            let end = ladder.0.1.max(ladder.1.1);

            if end - start + 1 >= self.min_strand_length {
                strand_regions.push((start, end));
            }
        }

        strand_regions
    }

    /// 查找β桥
    fn find_beta_bridges(
        &self,
        residues: &[Residue],
        hbonds: &[HydrogenBond],
    ) -> Vec<(usize, usize, bool)> {
        let mut bridges = Vec::new();

        for i in 0..residues.len() {
            for j in (i + 2)..residues.len() {
                // 检查平行桥
                let parallel_bridge = self.is_parallel_bridge(i, j, residues, hbonds);
                if parallel_bridge {
                    bridges.push((i, j, true));
                }

                // 检查反平行桥
                let antiparallel_bridge = self.is_antiparallel_bridge(i, j, residues, hbonds);
                if antiparallel_bridge {
                    bridges.push((i, j, false));
                }
            }
        }

        bridges
    }

    /// 检查平行桥
    fn is_parallel_bridge(
        &self,
        i: usize,
        j: usize,
        residues: &[Residue],
        hbonds: &[HydrogenBond],
    ) -> bool {
        if i == 0 || j + 1 >= residues.len() {
            return false;
        }

        let hbond1 = hbonds
            .iter()
            .any(|hb| hb.donor_idx == i - 1 && hb.acceptor_idx == j);
        let hbond2 = hbonds
            .iter()
            .any(|hb| hb.donor_idx == j && hb.acceptor_idx == i + 1);

        hbond1 && hbond2
    }

    /// 检查反平行桥
    fn is_antiparallel_bridge(
        &self,
        i: usize,
        j: usize,
        residues: &[Residue],
        hbonds: &[HydrogenBond],
    ) -> bool {
        let hbond1 = hbonds
            .iter()
            .any(|hb| hb.donor_idx == i && hb.acceptor_idx == j);
        let hbond2 = hbonds
            .iter()
            .any(|hb| hb.donor_idx == j && hb.acceptor_idx == i);

        if hbond1 && hbond2 {
            return true;
        }

        if i > 0 && j + 1 < residues.len() {
            let hbond3 = hbonds
                .iter()
                .any(|hb| hb.donor_idx == i - 1 && hb.acceptor_idx == j + 1);
            let hbond4 = hbonds
                .iter()
                .any(|hb| hb.donor_idx == j - 1 && hb.acceptor_idx == i + 1);

            hbond3 && hbond4
        } else {
            false
        }
    }

    /// 构建β梯子
    fn build_beta_ladders(
        &self,
        bridges: &[(usize, usize, bool)],
        num_residues: usize,
    ) -> Vec<((usize, usize), (usize, usize))> {
        let mut ladders = Vec::new();

        for &(i, j, is_parallel) in bridges {
            // 查找延伸的梯子
            if let Some(ladder) = self.extend_ladder(i, j, is_parallel, bridges, num_residues) {
                ladders.push(ladder);
            }
        }

        // 去重
        ladders.sort();
        ladders.dedup();
        ladders
    }

    /// 延伸梯子
    fn extend_ladder(
        &self,
        start_i: usize,
        start_j: usize,
        is_parallel: bool,
        bridges: &[(usize, usize, bool)],
        num_residues: usize,
    ) -> Option<((usize, usize), (usize, usize))> {
        let mut i = start_i;
        let mut j = start_j;
        let mut length = 1;

        // 向前延伸
        while i + 1 < num_residues && j + 1 < num_residues {
            let next_i = i + 1;
            let next_j = if is_parallel {
                j + 1
            } else {
                j.saturating_sub(1)
            };

            if bridges.contains(&(next_i, next_j, is_parallel)) {
                i = next_i;
                j = next_j;
                length += 1;
            } else {
                break;
            }
        }

        if length >= self.min_strand_length {
            let strand1 = (start_i, i);
            let strand2 = if is_parallel {
                (start_j, j)
            } else {
                (j, start_j) // 反平行时调整顺序
            };
            Some((strand1, strand2))
        } else {
            None
        }
    }

    /// 分配二级结构标签
    fn assign_ss_labels(
        &self,
        num_residues: usize,
        helices: &[(usize, usize)],
        strands: &[(usize, usize)],
    ) -> Vec<SecondaryStructure> {
        let mut labels = vec![SecondaryStructure::Coil; num_residues];

        // 优先分配螺旋(螺旋优先级高于β折叠)
        for &(start, end) in helices {
            for i in start..=end {
                if i < num_residues {
                    labels[i] = SecondaryStructure::Helix;
                }
            }
        }

        // 分配β折叠
        for &(start, end) in strands {
            for i in start..=end {
                if i < num_residues && labels[i] == SecondaryStructure::Coil {
                    labels[i] = SecondaryStructure::Sheet;
                }
            }
        }

        // 识别转角(基于氢键模式，这里简化处理)
        self.identify_turns(&mut labels);

        labels
    }

    /// 识别转角
    fn identify_turns(&self, labels: &mut [SecondaryStructure]) {
        // 简化的转角识别：在螺旋和折叠之间的短片段标记为转角
        let mut i = 0;
        while i < labels.len() {
            if labels[i] == SecondaryStructure::Coil {
                let start = i;
                while i < labels.len() && labels[i] == SecondaryStructure::Coil {
                    i += 1;
                }
                let length = i - start;
                if length >= 1 && length <= 4 {
                    for j in start..i {
                        labels[j] = SecondaryStructure::Turn;
                    }
                }
            } else {
                i += 1;
            }
        }
    }
}
