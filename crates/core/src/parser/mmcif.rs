use crate::parser::compute_secondary_structure::SecondaryStructureCalculator;
use crate::utils::vec_f16_scaled;
pub use crate::utils::{Logger, RustLogger};
use glam::Vec3;
use na_seq::AaIdent;
use na_seq::{AminoAcid, AtomTypeInRes, Element};
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct AtomGeneric {
    /// A unique identifier for this atom, within its molecule. This may originate from data in
    /// mmCIF files, Mol2, SDF files, etc.
    pub serial_number: u32,
    pub posit: Vec3,
    pub element: Element,
    /// This identifier will be unique within a given residue. For example, within an
    /// amino acid on a protein. Different residues will have different sets of these.
    /// e.g. "CG1", "CA", "O", "C", "HA", "CD", "C9" etc.
    pub type_in_res: Option<AtomTypeInRes>,
    /// There are too many variants of this (with different numbers) to use an enum effectively
    pub type_in_res_lipid: Option<String>,
    /// Used by Amber and other force fields to apply the correct molecular dynamics parameters for
    /// this atom.
    /// E.g. "c6", "ca", "n3", "ha", "h0" etc, as seen in Mol2 files from AMBER.
    /// e.g.: "ha": hydrogen attached to an aromatic carbon.
    /// "ho": hydrogen on a hydroxyl oxygen
    /// "n3": sp³ nitrogen with three substitutes
    /// "c6": sp² carbon in a pure six-membered aromatic ring (new in GAFF2; lets GAFF distinguish
    /// a benzene carbon from other aromatic caca carbons)
    /// For proteins, this appears to be the same as for `name`.
    pub force_field_type: Option<String>,
    /// An atom-centered electric charge, used in molecular dynamics simulations. In elementary charge units.
    /// These are sometimes loaded from Amber-provided Mol2 or SDF files, and sometimes added after.
    /// We get partial charge for ligands from (e.g. Amber-provided) Mol files, so we load it from the atom, vice
    /// the loaded FF params. Convert to appropriate units prior to running dynamics.
    pub partial_charge: Option<f32>,
    /// Indicates, in proteins, that the atom isn't part of an amino acid. E.g., water or
    /// ligands.
    pub hetero: bool,
    pub occupancy: Option<f32>,
    /// Used by mmCIF files to store alternate conformations. If this isn't None, there may
    /// be, for example, an "A" and "B" variant of this atom at slightly different positions.
    pub alt_conformation_id: Option<String>,
}

impl Display for AtomGeneric {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ff_type = match &self.force_field_type {
            Some(f) => f,
            None => "None",
        };

        let q = match &self.partial_charge {
            Some(q_) => format!("{q_:.3}"),
            None => "None".to_string(),
        };

        write!(
            f,
            "Atom {}: {}, {}. {:?}, ff: {ff_type}, q: {q}",
            self.serial_number,
            self.element.to_letter(),
            self.posit,
            self.type_in_res,
        )?;

        if self.hetero {
            write!(f, ", Het")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChainGeneric {
    pub id: String,
    // todo: Do we want both residues and atoms stored here? It's an overconstraint.
    /// Serial number
    pub residue_sns: Vec<u32>,
    /// Serial number
    pub atom_sns: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct ResidueGeneric {
    /// We use serial number of display, search etc, and array index to select. Residue serial number is not
    /// unique in the molecule; only in the chain.
    pub serial_number: u32,
    pub res_type: ResidueType,
    /// Serial number
    pub atom_sns: Vec<u32>,
    pub end: ResidueEnd,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResidueEnd {
    Internal,
    NTerminus,
    CTerminus,
    /// Not part of a protein/polypeptide.
    Hetero,
}

#[derive(Clone, Debug)]
pub struct MmCif {
    pub ident: String,
    pub metadata: HashMap<String, String>,
    pub atoms: Vec<AtomGeneric>,
    // This is sometimes included in mmCIF files, although seems to be absent
    // from most (all?) on RCSB PDB.
    // pub bonds: Vec<BondGeneric>,
    pub chains: Vec<ChainGeneric>,
    pub residues: Vec<ResidueGeneric>,
    pub secondary_structure: Vec<BackboneSS>,
    pub experimental_method: Option<ExperimentalMethod>,
}

impl MmCif {
    pub fn new(text: &str) -> io::Result<Self> {
        // todo: For these `new` methods in general that take a &str param: Should we use
        // todo R: Reed + Seek instead, and pass a Cursor or File object? Probably doesn't matter.
        // todo Either way, we should keep it consistent between the files.

        // todo: This is far too slow.

        let mut metadata = HashMap::<String, String>::new();
        let mut atoms = Vec::<AtomGeneric>::new();
        let mut residues = Vec::<ResidueGeneric>::new();
        let mut chains = Vec::<ChainGeneric>::new();
        let mut res_idx = HashMap::<(String, u32), usize>::new();
        let mut chain_idx = HashMap::<String, usize>::new();

        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        let n = lines.len();

        let mut experimental_method: Option<ExperimentalMethod> = None;

        let method_re = Regex::new(r#"^_exptl\.method\s+['"]([^'"]+)['"]\s*$"#).unwrap();

        while i < n {
            let mut line = lines[i].trim();
            if line.is_empty() {
                i += 1;
                continue;
            }

            if let Some(caps) = method_re.captures(line)
                && let Ok(m) = caps[1].to_string().parse()
            {
                experimental_method = Some(m);
            }

            if line == "loop_" {
                i += 1;
                let mut headers = Vec::<&str>::new();
                while i < n {
                    line = lines[i].trim();
                    if line.starts_with('_') {
                        headers.push(line);
                        i += 1;
                    } else {
                        break;
                    }
                }

                // If not an atom loops, skip first rows.
                if !headers
                    .first()
                    .is_some_and(|h| h.starts_with("_atom_site."))
                {
                    while i < n {
                        line = lines[i].trim();
                        if line == "#" || line == "loop_" || line.starts_with('_') {
                            break;
                        }
                        i += 1;
                    }
                    continue;
                }

                let col = |tag: &str| -> io::Result<usize> {
                    headers.iter().position(|h| *h == tag).ok_or_else(|| {
                        io::Error::new(ErrorKind::InvalidData, format!("mmCIF missing {tag}"))
                    })
                };
                let het = col("_atom_site.group_PDB")?;
                let c_id = col("_atom_site.id")?;
                let c_x = col("_atom_site.Cartn_x")?;
                let c_y = col("_atom_site.Cartn_y")?;
                let c_z = col("_atom_site.Cartn_z")?;
                let c_el = col("_atom_site.type_symbol")?;
                let c_name = col("_atom_site.label_atom_id")?;
                let c_alt_id = col("_atom_site.label_alt_id")?;
                let c_res = col("_atom_site.label_comp_id")?;
                let c_chain = col("_atom_site.label_asym_id")?;
                let c_res_sn = col("_atom_site.label_seq_id")?;
                let c_occ = col("_atom_site.occupancy")?;

                while i < n {
                    line = lines[i].trim();
                    if line.is_empty() || line == "#" || line == "loop_" || line.starts_with('_') {
                        break;
                    }
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() < headers.len() {
                        i += 1;
                        continue;
                    }

                    // Atom lines.
                    let hetero = fields[het].trim() == "HETATM";

                    let serial_number = fields[c_id].parse::<u32>().unwrap_or(0);
                    let x = fields[c_x].parse::<f64>().unwrap_or(0.0);
                    let y = fields[c_y].parse::<f64>().unwrap_or(0.0);
                    let z = fields[c_z].parse::<f64>().unwrap_or(0.0);

                    let element = Element::from_letter(fields[c_el])?;
                    let atom_name = fields[c_name];

                    let alt_conformation_id = if fields[c_alt_id] == "." {
                        None
                    } else {
                        Some(fields[c_alt_id].to_string())
                    };

                    let type_in_res = if hetero {
                        if !atom_name.is_empty() {
                            Some(AtomTypeInRes::Hetero(atom_name.to_string()))
                        } else {
                            None
                        }
                    } else {
                        AtomTypeInRes::from_str(atom_name).ok()
                    };

                    let occ = match fields[c_occ] {
                        "?" | "." => None,
                        v => v.parse().ok(),
                    };

                    atoms.push(AtomGeneric {
                        serial_number,
                        posit: Vec3::new(x as f32, y as f32, z as f32),
                        element,
                        type_in_res,
                        occupancy: occ,
                        hetero,
                        alt_conformation_id,
                        ..Default::default()
                    });

                    // --------- Residue / Chain bookkeeping -----------
                    let res_sn = fields[c_res_sn].parse::<u32>().unwrap_or(0);
                    let chain_id = fields[c_chain];
                    let res_key = (chain_id.to_string(), res_sn);

                    // Residues
                    let r_i = *res_idx.entry(res_key.clone()).or_insert_with(|| {
                        let idx = residues.len();
                        residues.push(ResidueGeneric {
                            serial_number: res_sn,
                            res_type: ResidueType::from_str(fields[c_res]),
                            atom_sns: Vec::new(),
                            end: ResidueEnd::Internal, // We update this after.
                        });
                        idx
                    });
                    residues[r_i].atom_sns.push(serial_number);

                    // Chains
                    let c_i = *chain_idx.entry(chain_id.to_string()).or_insert_with(|| {
                        let idx = chains.len();
                        chains.push(ChainGeneric {
                            id: chain_id.to_string(),
                            residue_sns: Vec::new(),
                            atom_sns: Vec::new(),
                        });
                        idx
                    });
                    chains[c_i].atom_sns.push(serial_number);
                    if !chains[c_i].residue_sns.contains(&res_sn) {
                        chains[c_i].residue_sns.push(res_sn);
                    }

                    i += 1;
                }
                continue; // outer while will handle terminator line
            }

            if line.starts_with('_') {
                if let Some((tag, val)) = line.split_once(char::is_whitespace) {
                    metadata.insert(
                        tag.to_string(),
                        val.trim_matches('\'').to_string().trim().to_string(),
                    );
                } else {
                    metadata.insert(line.to_string().trim().to_string(), String::new());
                }
            }

            i += 1; // advance to next top-level line
        }

        // Populate the residue end, now that we know when the last non-het one is.
        {
            let mut last_non_het = 0;
            for (i, res) in residues.iter().enumerate() {
                match res.res_type {
                    ResidueType::AminoAcid(_) => last_non_het = i,
                    _ => break,
                }
            }

            for (i, res) in residues.iter_mut().enumerate() {
                let mut end = ResidueEnd::Internal;

                // Match arm won't work due to non-constant arms, e.g. non_hetero?
                if i == 0 {
                    end = ResidueEnd::NTerminus;
                } else if i == last_non_het {
                    end = ResidueEnd::CTerminus;
                }

                match res.res_type {
                    ResidueType::AminoAcid(_) => (),
                    _ => end = ResidueEnd::Hetero,
                }

                res.end = end;
            }
        }

        let ident = metadata
            .get("_struct.entry_id")
            .or_else(|| metadata.get("_entry.id"))
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string())
            .trim()
            .to_owned();

        // let mut cursor = Cursor::new(text);

        // let ss_load = Instant::now();
        // todo: Integraet this so it's not taking a second line loop through the whole file.
        // todo: It'll be faster this way.
        // todo: Regardless of that, this SS loading is going very slowly. Fix it.
        // let (secondary_structure, experimental_method) = load_ss_method(&mut cursor)?;

        // let ss_load_time = ss_load.elapsed();
        let secondary_structure = Vec::new();

        Ok(Self {
            ident,
            metadata,
            atoms,
            chains,
            residues,
            secondary_structure,
            experimental_method,
        })
    }

    // todo: QC this.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;

        fn quote_if_needed(s: &str) -> String {
            if s.is_empty() || s.chars().any(|c| c.is_whitespace()) {
                let esc = s.replace('\'', "''");
                format!("'{}'", esc)
            } else {
                s.to_string()
            }
        }

        let ident = {
            let id = self.ident.trim();
            if id.is_empty() { "UNKNOWN" } else { id }
        };

        // Header + minimal metadata
        writeln!(file, "data_{}", ident)?;
        writeln!(file, "_struct.entry_id {}", quote_if_needed(ident))?;
        if let Some(m) = &self.experimental_method {
            writeln!(file, "_exptl.method {}", quote_if_needed(&m.to_string()))?;
        }
        for (k, v) in &self.metadata {
            if k == "_struct.entry_id" || k == "_entry.id" || k == "_exptl.method" {
                continue;
            }
            if k.starts_with('_') {
                writeln!(file, "{} {}", k, quote_if_needed(v))?;
            }
        }
        writeln!(file, "#")?;

        // Build lookups for atom → residue and atom → chain
        let mut atom_to_res = HashMap::<u32, u32>::new();
        for r in &self.residues {
            for &sn in &r.atom_sns {
                atom_to_res.insert(sn, r.serial_number);
            }
        }
        let mut res_map = HashMap::<u32, &ResidueGeneric>::new();
        for r in &self.residues {
            res_map.insert(r.serial_number, r);
        }
        let mut atom_to_chain = HashMap::<u32, &str>::new();
        for c in &self.chains {
            for &sn in &c.atom_sns {
                atom_to_chain.insert(sn, &c.id);
            }
        }

        // _atom_site loop (matches the columns the loader reads)
        writeln!(file, "loop_")?;
        writeln!(file, "_atom_site.group_PDB")?;
        writeln!(file, "_atom_site.id")?;
        writeln!(file, "_atom_site.Cartn_x")?;
        writeln!(file, "_atom_site.Cartn_y")?;
        writeln!(file, "_atom_site.Cartn_z")?;
        writeln!(file, "_atom_site.type_symbol")?;
        writeln!(file, "_atom_site.label_atom_id")?;
        writeln!(file, "_atom_site.label_comp_id")?;
        writeln!(file, "_atom_site.label_asym_id")?;
        writeln!(file, "_atom_site.label_seq_id")?;
        writeln!(file, "_atom_site.occupancy")?;

        for a in &self.atoms {
            let group = if a.hetero { "HETATM" } else { "ATOM" };
            let sym = a.element.to_string();
            let atom_name = match &a.type_in_res {
                Some(na_seq::AtomTypeInRes::Hetero(n)) => n.clone(),
                Some(t) => t.to_string(),
                None => sym.clone(),
            };
            let res_sn = *atom_to_res.get(&a.serial_number).unwrap_or(&0u32);
            let (res_name, chain_id) = if let Some(r) = res_map.get(&res_sn) {
                (
                    r.res_type.to_string(),
                    atom_to_chain.get(&a.serial_number).copied().unwrap_or("A"),
                )
            } else {
                (
                    "UNK".to_string(),
                    atom_to_chain.get(&a.serial_number).copied().unwrap_or("A"),
                )
            };
            let occ_s = match a.occupancy {
                Some(o) => format!("{:.2}", o),
                None => "?".to_string(),
            };

            writeln!(
                file,
                "{} {} {:.3} {:.3} {:.3} {} {} {} {} {}",
                group,
                a.serial_number,
                a.posit.x,
                a.posit.y,
                a.posit.z,
                quote_if_needed(&sym),
                quote_if_needed(&atom_name),
                quote_if_needed(&res_name),
                quote_if_needed(chain_id),
                res_sn,
            )?;
            writeln!(file, "{}", occ_s)?;
        }

        writeln!(file, "#")?;
        Ok(())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let data_str = fs::read_to_string(path)?;
        Self::new(&data_str)
    }

    // Download Load from DrugBank from the RCSB Protein Data Bank. (PDB)
    // pub fn load_rcsb(ident: &str) -> io::Result<Self> {
    //     unimplemented!()
    //     // let data_str =
    //     //     rcsb::load_cif(ident).map_err(|e| io::Error::other(format!("Error loading: {e:?}")))?;
    //     // Self::new(&data_str)
    // }
}

#[derive(Clone, Debug)]
/// See note elsewhere regarding serial numbers vs indices: In your downstream applications, you may
/// wish to convert sns to indices, for faster operations.
pub struct BackboneSS {
    /// Atom serial numbers.
    pub start_sn: u32,
    pub end_sn: u32,
    pub sec_struct: SecondaryStructure,
}
#[derive(Clone, Copy, PartialEq, Debug)]
/// The method used to find a given molecular structure. This data is present in mmCIF files
/// as the `_exptl.method` field.
pub enum ExperimentalMethod {
    XRayDiffraction,
    ElectronDiffraction,
    NeutronDiffraction,
    /// i.e. Cryo-EM
    ElectronMicroscopy,
    SolutionNmr,
}

impl ExperimentalMethod {
    /// E.g. for displaying in the space-constrained UI.
    pub fn to_str_short(&self) -> String {
        match self {
            Self::XRayDiffraction => "X-ray",
            Self::NeutronDiffraction => "ND",
            Self::ElectronDiffraction => "ED",
            Self::ElectronMicroscopy => "EM",
            Self::SolutionNmr => "NMR",
        }
        .to_owned()
    }
}

impl Display for ExperimentalMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let val = match self {
            Self::XRayDiffraction => "X-Ray diffraction",
            Self::NeutronDiffraction => "Neutron diffraction",
            Self::ElectronDiffraction => "Electron diffraction",
            Self::ElectronMicroscopy => "Electron microscopy",
            Self::SolutionNmr => "Solution NMR",
        };
        write!(f, "{val}")
    }
}

impl FromStr for ExperimentalMethod {
    type Err = io::Error;

    /// Parse an mmCIF‐style method string into an ExperimentalMethod.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.to_lowercase();
        let s = normalized.trim();
        let method = match s {
            "x-ray diffraction" => ExperimentalMethod::XRayDiffraction,
            "neutron diffraction" => ExperimentalMethod::NeutronDiffraction,
            "electron diffraction" => ExperimentalMethod::ElectronDiffraction,
            "electron microscopy" => ExperimentalMethod::ElectronMicroscopy,
            "solution nmr" => ExperimentalMethod::SolutionNmr,
            other => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Error parsing experimental method: {other}"),
                ));
            }
        };
        Ok(method)
    }
}

pub struct ParserOptions {}
pub fn parse_mmcif(sdf: &str, options: Option<&ParserOptions>) -> MmCif {
    _parse_mmcif(sdf, options, RustLogger)
}

pub fn _parse_mmcif(
    mmcif_str: &str,
    _options: Option<&ParserOptions>,
    _logger: impl Logger,
) -> MmCif {
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

    #[serde(skip)]
    ss_cache: OnceCell<Vec<SecondaryStructure>>,
}

impl Chain {
    pub fn get_ss(&self) -> &Vec<SecondaryStructure> {
        self.ss_cache.get_or_init(|| {
            let calculator = SecondaryStructureCalculator::new();
            calculator.compute_secondary_structure(&self.residues)
        })
    }

    pub fn new(id: String, residues: Vec<Residue>) -> Self {
        Self {
            id,
            residues,
            ss_cache: OnceCell::new(), // 初始化私有缓存
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Residue {
    #[serde(with = "aa_serde")]
    pub residue_type: AminoAcid, // e.g. "ALA", "GLY"
    // pub residue_type: ResidueType, // e.g. "ALA", "GLY"
    pub sns: usize, // PDB numbering or sequential

    // Minimum for cartoon backbone
    #[serde(with = "vec_f16_scaled")]
    pub c: Vec3, // or pseudo-CB for glycine
    #[serde(with = "vec_f16_scaled")]
    pub n: Vec3, // C-alpha coordinates
    #[serde(with = "vec_f16_scaled")]
    pub ca: Vec3, // C-alpha coordinates
    #[serde(with = "vec_f16_scaled")]
    pub o: Vec3, // C-alpha coordinates
    pub h: Option<Vec3>, // C-alpha coordinates

    // Secondary structure tag
    pub ss: Option<SecondaryStructure>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryStructure {
    Helix,
    Sheet,
    Coil,
    Turn,
}

mod aa_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(aa: &AminoAcid, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&aa.to_str(AaIdent::OneLetter))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<AminoAcid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(d)?;
        AminoAcid::from_str(&name)
            .map_err(|_| serde::de::Error::custom(format!("Invalid amino acid string: {}", name)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ResidueType {
    #[serde(with = "aa_serde")]
    AminoAcid(AminoAcid),
    Water,
    Other(String),
}

impl From<&ResidueType> for ResidueType {
    fn from(res_type: &ResidueType) -> Self {
        match res_type {
            ResidueType::AminoAcid(a) => ResidueType::AminoAcid(*a),
            ResidueType::Water => ResidueType::Water,
            ResidueType::Other(s) => ResidueType::Other(s.clone()),
        }
    }
}

impl Display for ResidueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match &self {
            ResidueType::Other(n) => n.clone(),
            ResidueType::Water => "Water".to_string(),
            ResidueType::AminoAcid(aa) => aa.to_string(),
        };

        write!(f, "{name}")
    }
}

impl Default for ResidueType {
    fn default() -> Self {
        Self::Other(String::new())
    }
}

impl ResidueType {
    /// Parses from the "name" field in common text-based formats lik CIF, PDB, and PDBQT.
    pub fn from_str(name: &str) -> Self {
        if name.to_uppercase() == "HOH" {
            ResidueType::Water
        } else {
            match AminoAcid::from_str(name) {
                Ok(aa) => ResidueType::AminoAcid(aa),
                Err(_) => ResidueType::Other(name.to_owned()),
            }
        }
    }
}
