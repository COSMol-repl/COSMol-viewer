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
    pub bonds: Vec<BondGeneric>,
    pub chains: Vec<ChainGeneric>,
    pub residues: Vec<ResidueGeneric>,
    pub pharmacophore_features: Vec<PharmacaphoreFeatures>,
}

impl Sdf {
    /// From a string of an SDF text file.
    pub fn new(text: &str) -> io::Result<Self> {
        let lines: Vec<&str> = text.lines().collect();

        // SDF files typically have at least 4 lines before the atom block:
        //   1) A title or identifier
        //   2) Usually blank or comments
        //   3) Often blank or comments
        //   4) "counts" line: e.g. " 50  50  0  ..." for V2000
        if lines.len() < 4 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Not enough lines to parse an SDF header",
            ));
        }

        // todo: Incorporate more cols A/R.
        // After element:
        // Mass difference (0, unless an isotope)
        // Charge (+1 for cation etc)
        // Stereo, valence, other flags

        // todo: Do bonds too
        // first atom index
        // second atom index
        // 1 for single, 2 for double etc
        // 0 for no stereochemistry, 1=up, 6=down etc
        // Other properties: Bond topology, reaction center flags etc. Usually 0

        // This is the "counts" line, e.g. " 50 50  0  0  0  0  0  0  0999 V2000"
        let counts_line = lines[3];
        let counts_cols: Vec<&str> = counts_line.split_whitespace().collect();

        if counts_cols.len() < 2 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Counts line doesn't have enough fields",
            ));
        }

        // Typically, the first number is the number of atoms (natoms)
        // and the second number is the number of bonds (nbonds).
        let n_atoms = counts_cols[0].parse::<usize>().map_err(|_| {
            io::Error::new(ErrorKind::InvalidData, "Could not parse number of atoms")
        })?;
        let n_bonds = counts_cols[1].parse::<usize>().map_err(|_| {
            io::Error::new(ErrorKind::InvalidData, "Could not parse number of bonds")
        })?;

        // Now read the next 'natoms' lines as the atom block.
        // Each line usually looks like:
        //   X Y Z Element ??? ??? ...
        //   e.g. "    1.4386   -0.8054   -0.4963 O   0  0  0  0  0  0  0  0  0  0  0  0"
        //

        let first_atom_line = 4;
        let last_atom_line = first_atom_line + n_atoms;
        let first_bond_line = last_atom_line;
        let last_bond_line = first_bond_line + n_bonds;

        if lines.len() < last_atom_line {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Not enough lines for the declared atom block",
            ));
        }

        let mut atoms = Vec::with_capacity(n_atoms);

        let pharmacophore_features: Vec<PharmacaphoreFeatures> = Vec::new();

        for i in first_atom_line..last_atom_line {
            let line = lines[i];
            let cols: Vec<&str> = line.split_whitespace().collect();

            if cols.len() < 4 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Atom line {i} does not have enough columns"),
                ));
            }

            let x = cols[0].parse::<f64>().map_err(|_| {
                io::Error::new(ErrorKind::InvalidData, "Could not parse X coordinate")
            })?;
            let y = cols[1].parse::<f64>().map_err(|_| {
                io::Error::new(ErrorKind::InvalidData, "Could not parse Y coordinate")
            })?;
            let z = cols[2].parse::<f64>().map_err(|_| {
                io::Error::new(ErrorKind::InvalidData, "Could not parse Z coordinate")
            })?;
            let element = cols[3];

            atoms.push(AtomGeneric {
                // SDF doesn't explicitly include incices.
                serial_number: (i - first_atom_line) as u32 + 1,
                posit: Vec3 {
                    x: x as f32,
                    y: y as f32,
                    z: z as f32,
                }, // or however you store coordinates
                element: Element::from_letter(element)?,
                hetero: true,
                ..Default::default()
            });
        }

        let mut bonds = Vec::with_capacity(n_bonds);
        for i in first_bond_line..last_bond_line {
            let line = lines[i];
            let cols: Vec<&str> = line.split_whitespace().collect();

            if cols.len() < 3 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Bond line {i} does not have enough columns"),
                ));
            }

            let atom_0_sn = cols[0].parse::<u32>().map_err(|_| {
                io::Error::new(ErrorKind::InvalidData, "Could not parse bond atom 0")
            })?;
            let atom_1_sn = cols[1].parse::<u32>().map_err(|_| {
                io::Error::new(ErrorKind::InvalidData, "Could not parse bond atom 1")
            })?;

            let bond_type = BondType::from_str(cols[2])?;

            bonds.push(BondGeneric {
                atom_0_sn,
                atom_1_sn,
                bond_type,
            })
        }

        // Look for a molecule identifier in the file. Check for either
        // "> <PUBCHEM_COMPOUND_CID>" or "> <DATABASE_ID>" and take the next nonempty line.
        let mut _pubchem_cid = None;
        let mut _drugbank_id = None;

        for (i, line) in lines.iter().enumerate() {
            if line.contains("> <PUBCHEM_COMPOUND_CID>")
                && let Some(value_line) = lines.get(i + 1)
            {
                let value = value_line.trim();
                if let Ok(v) = value.parse::<u32>() {
                    _pubchem_cid = Some(v);
                }
            }
            if line.contains("> <DATABASE_ID>")
                && let Some(value_line) = lines.get(i + 1)
            {
                let value = value_line.trim();
                if !value.is_empty() {
                    _drugbank_id = Some(value.to_string());
                }
            }
        }

        let ident = lines[0].trim().to_string();
        // We observe that on at least some DrugBank files, this line
        // is the PubChem ID, even if the PUBCHEM_COMPOUND_CID line is omitted.
        if let Ok(v) = lines[0].parse::<u32>() {
            _pubchem_cid = Some(v);
        }

        // We could now skip over the bond lines if we want:
        //   let first_bond_line = last_atom_ line;
        //   let last_bond_line = first_bond_line + nbonds;
        // etc.
        // Then we look for "M  END" or the data fields, etc.

        // For now, just return the Sdf with the atoms we parsed:

        let mut chains = Vec::new();
        let mut residues = Vec::new();

        // let atom_indices: Vec<usize> = (0..atoms.len()).collect();
        let atom_sns: Vec<_> = atoms.iter().map(|a| a.serial_number).collect();

        residues.push(ResidueGeneric {
            serial_number: 0,
            res_type: ResidueType::Other("Unknown".to_string()),
            atom_sns: atom_sns.clone(),
            end: ResidueEnd::Hetero,
        });

        chains.push(ChainGeneric {
            id: "A".to_string(),
            residue_sns: vec![0],
            atom_sns,
        });

        Ok(Self {
            ident,
            metadata: HashMap::new(),
            atoms,
            chains,
            residues,
            bonds,
            pharmacophore_features,
        })
    }
}
