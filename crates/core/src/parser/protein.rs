pub use crate::utils::{Logger, RustLogger};
use bio_files::MmCif;
use glam::Vec3;
use na_seq::AminoAcid;
use serde::{Deserialize, Serialize};

pub struct ParserOptions {}

pub fn parse_mmcif(sdf: &str, options: Option<&ParserOptions>) -> MmCif {
    _parse_mmcif(sdf, options, RustLogger)
}

pub fn _parse_mmcif(
    mmcif_str: &str,
    options: Option<&ParserOptions>,
    _logger: impl Logger,
) -> MmCif {
    use bio_files::MmCif;

    let mmcif = MmCif::new(mmcif_str);

    match mmcif {
        Ok(mmcif) => mmcif,
        Err(err) => {
            _logger.error(&format!("Error parsing MMCIF: {}", err));
            panic!("Error parsing MMCIF: {}", err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chain {
    pub id: String,
    pub residues: Vec<Residue>,
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
