use crate::parser::utils::{
    AtomGeneric, BondGeneric, BondType, ChainGeneric, PharmacaphoreFeatures, ResidueEnd,
    ResidueGeneric, ResidueType,
};
pub use crate::utils::{Logger, RustLogger};
use glam::Vec3;
use na_seq::Element;
use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Sdf {
    pub ident: String,
    pub metadata: HashMap<String, String>,
    pub atoms: Vec<AtomGeneric>,
    pub atoms_weight: Option<Vec<Option<f32>>>,
    pub bonds: Vec<BondGeneric>,
    pub chains: Vec<ChainGeneric>,
    pub residues: Vec<ResidueGeneric>,
    pub pharmacophore_features: Vec<PharmacaphoreFeatures>,
}

impl Sdf {
    pub fn new(text: &str) -> io::Result<Self> {
        let lines: Vec<&str> = text.lines().collect();

        if lines.len() < 4 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Not enough lines for SDF header",
            ));
        }

        let ident = lines[0].trim().to_string();
        let counts_line = lines[3];

        let is_v3000 = counts_line.contains("V3000");
        let is_v2000 = counts_line.contains("V2000");

        if !is_v2000 && !is_v3000 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unknown SDF version (neither V2000 nor V3000)",
            ));
        }

        let mut atoms = Vec::new();
        let mut bonds = Vec::new();

        let mut atoms_weight: Option<Vec<Option<f32>>> = None;

        if is_v2000 {
            // ============================
            // V2000 parsing
            // ============================

            let n_atoms = counts_line
                .get(0..3)
                .unwrap_or("")
                .trim()
                .parse::<usize>()
                .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid atom count"))?;

            let n_bonds = counts_line
                .get(3..6)
                .unwrap_or("")
                .trim()
                .parse::<usize>()
                .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid bond count"))?;

            let first_atom_line = 4;
            let last_atom_line = first_atom_line + n_atoms;
            let first_bond_line = last_atom_line;
            let last_bond_line = first_bond_line + n_bonds;

            // -------- atoms --------
            for i in first_atom_line..last_atom_line {
                let cols: Vec<&str> = lines[i].split_whitespace().collect();
                if cols.len() < 4 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid atom line at {}", i),
                    ));
                }

                let x = cols[0].parse::<f64>().unwrap();
                let y = cols[1].parse::<f64>().unwrap();
                let z = cols[2].parse::<f64>().unwrap();
                let element_str = cols[3];

                let element = match Element::from_letter(element_str) {
                    Ok(element) => element,
                    Err(_) => Element::Other,
                };

                atoms.push(AtomGeneric {
                    serial_number: (i - first_atom_line + 1) as u32,
                    posit: Vec3::new(x as f32, y as f32, z as f32),
                    element: element,
                    hetero: true,
                    ..Default::default()
                });
            }

            // -------- bonds --------
            for i in first_bond_line..last_bond_line {
                let line = lines[i];

                let atom_0_sn: u32 =
                    line.get(0..3).unwrap_or("").trim().parse().map_err(|_| {
                        io::Error::new(ErrorKind::InvalidData, "Invalid bond atom 1")
                    })?;

                let atom_1_sn: u32 =
                    line.get(3..6).unwrap_or("").trim().parse().map_err(|_| {
                        io::Error::new(ErrorKind::InvalidData, "Invalid bond atom 2")
                    })?;

                let bond_type_raw = line.get(6..9).unwrap_or("").trim();
                let bond_type = match BondType::from_str(bond_type_raw) {
                    Ok(bond_type) => bond_type,
                    Err(_) => BondType::Unknown,
                };

                bonds.push(BondGeneric {
                    atom_0_sn,
                    atom_1_sn,
                    bond_type,
                });
            }
        } else {
            // ============================
            // V3000 parsing
            // ============================

            let mut i = 4;

            // --- find counts ---
            let mut n_atoms = 0usize;
            let mut n_bonds = 0usize;
            let mut atoms_weight_: Vec<Option<f32>> = Vec::new();

            while i < lines.len() {
                let line = lines[i];
                if line.starts_with("M  V30 COUNTS") {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    n_atoms = cols[3].parse().unwrap();
                    n_bonds = cols[4].parse().unwrap();
                }
                if line.starts_with("M  V30 BEGIN ATOM") {
                    i += 1;
                    break;
                }
                i += 1;
            }

            // --- atoms ---
            for _ in 0..n_atoms {
                let cols: Vec<&str> = lines[i].split_whitespace().collect();
                // M  V30 idx element x y z ...
                let serial_number = cols[2].parse::<u32>().unwrap();
                let element_str = cols[3];
                let x = cols[4].parse::<f64>().unwrap();
                let y = cols[5].parse::<f64>().unwrap();
                let z = cols[6].parse::<f64>().unwrap();

                // ---- parse WEIGHT if exists ----
                let mut weight: Option<f32> = None;

                for col in cols.iter().skip(7) {
                    if let Some(v) = col.strip_prefix("WEIGHT=") {
                        if let Ok(w) = v.parse::<f32>() {
                            weight = Some(w);
                        }
                    }
                }

                let element = match Element::from_letter(element_str) {
                    Ok(element) => element,
                    Err(_) => Element::Other,
                };

                atoms.push(AtomGeneric {
                    serial_number,
                    posit: Vec3::new(x as f32, y as f32, z as f32),
                    element: element,
                    hetero: true,
                    ..Default::default()
                });

                atoms_weight_.push(weight);

                i += 1;
            }

            atoms_weight = Some(atoms_weight_);

            // --- find bond begin ---
            while i < lines.len() && !lines[i].starts_with("M  V30 BEGIN BOND") {
                i += 1;
            }
            i += 1;

            // --- bonds ---
            for _ in 0..n_bonds {
                let cols: Vec<&str> = lines[i].split_whitespace().collect();
                // M  V30 idx type a1 a2
                let bond_type = match BondType::from_str(cols[3]) {
                    Ok(bond_type) => bond_type,
                    Err(_) => BondType::Unknown,
                };
                let atom_0_sn = cols[4].parse::<u32>().unwrap();
                let atom_1_sn = cols[5].parse::<u32>().unwrap();

                bonds.push(BondGeneric {
                    atom_0_sn,
                    atom_1_sn,
                    bond_type,
                });

                i += 1;
            }
        }

        // ============================
        // Unified residue / chain
        // ============================

        let atom_sns: Vec<u32> = atoms.iter().map(|a| a.serial_number).collect();

        let residues = vec![ResidueGeneric {
            serial_number: 0,
            res_type: ResidueType::Other("Unknown".to_string()),
            atom_sns: atom_sns.clone(),
            end: ResidueEnd::Hetero,
        }];

        let chains = vec![ChainGeneric {
            id: "A".to_string(),
            residue_sns: vec![0],
            atom_sns,
        }];

        Ok(Self {
            ident,
            metadata: HashMap::new(),
            atoms_weight,
            atoms,
            bonds,
            chains,
            residues,
            pharmacophore_features: Vec::new(),
        })
    }
}
