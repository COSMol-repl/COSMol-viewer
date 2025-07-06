#[derive(Debug, Clone)]
pub struct Atom {
    pub atom: String,
    pub elem: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub serial: usize,
    pub index: usize,
    pub hetflag: bool,
    pub bonds: Vec<usize>,
    pub bond_order: Vec<f32>,
    pub properties: std::collections::HashMap<String, String>,
}

pub type Molecule = Vec<Atom>;
pub type MoleculeData = Vec<Molecule>;

#[derive(Default)]
pub struct ParserOptions {
    pub keep_h: bool,
    pub multimodel: bool,
    pub onemol: bool,
}

pub fn parse_sdf(sdf: &str, options: &ParserOptions) -> MoleculeData {
    let lines: Vec<&str> = sdf.lines().collect();
    if lines.len() > 3 && lines[3].len() > 38 {
        let version = lines[3][34..39].trim();
        match version {
            "V3000" => parse_v3000(lines, options),
            _ => parse_v2000(lines, options),
        }
    } else {
        vec![vec![]]
    }
}

fn parse_v2000(mut lines: Vec<&str>, options: &ParserOptions) -> MoleculeData {
    let model_count = count_models(&lines);
    // 多个分子但用户没开启
    if model_count > 0 && !options.multimodel {
        panic!(
            "Found multiple molecules but 'multimodel' is false. Please enable 'multimodel = true' to parse all molecules."
        );
    }

    // 用户开启了但其实只有一个
    if model_count == 0 && options.multimodel {
        panic!(
            "Only one molecule found, but 'multimodel = true' was set. Consider setting 'multimodel = false' to avoid confusion."
        );
    }

    let mut molecules = vec![vec![]];
    let mut current = 0;

    while lines.len() >= 4 {
        let header = lines[3];
        let atom_count = header[0..3].trim().parse::<usize>().unwrap_or(0);
        let bond_count = header[3..6].trim().parse::<usize>().unwrap_or(0);

        if atom_count == 0 || lines.len() < 4 + atom_count + bond_count {
            break;
        }

        let mut serial_to_index = vec![None; atom_count];
        let mut offset = 4;
        let start = molecules[current].len();

        for i in 0..atom_count {
            let line = lines[offset + i];
            let elem = line[31..34].trim();
            let elem_cap = capitalize(elem);
            if elem_cap != "H" || options.keep_h {
                let atom = Atom {
                    atom: elem_cap.clone(),
                    elem: elem_cap,
                    x: line[0..10].trim().parse().unwrap_or(0.0),
                    y: line[10..20].trim().parse().unwrap_or(0.0),
                    z: line[20..30].trim().parse().unwrap_or(0.0),
                    serial: start + i,
                    index: molecules[current].len(),
                    hetflag: true,
                    bonds: vec![],
                    bond_order: vec![],
                    properties: std::collections::HashMap::new(),
                };
                serial_to_index[i] = Some(molecules[current].len());
                molecules[current].push(atom);
            }
        }

        offset += atom_count;

        for i in 0..bond_count {
            let line = lines[offset + i];
            let from = line[0..3]
                .trim()
                .parse::<usize>()
                .unwrap_or(0)
                .saturating_sub(1);
            let to = line[3..6]
                .trim()
                .parse::<usize>()
                .unwrap_or(0)
                .saturating_sub(1);
            let order = line[6..].trim().parse::<f32>().unwrap_or(1.0);
            if let (Some(f), Some(t)) = (
                serial_to_index.get(from).and_then(|x| *x),
                serial_to_index.get(to).and_then(|x| *x),
            ) {
                molecules[current][f].bonds.push(t);
                molecules[current][f].bond_order.push(order);
                molecules[current][t].bonds.push(f);
                molecules[current][t].bond_order.push(order);
            }
        }

        let mut next_offset = offset + bond_count;
        if options.multimodel {
            if !options.onemol {
                molecules.push(vec![]);
                current += 1;
            }
            while next_offset < lines.len() && lines[next_offset] != "$$$$" {
                next_offset += 1;
            }
            lines.drain(0..=next_offset);
        } else {
            break;
        }
    }

    molecules
}

fn parse_v3000(mut lines: Vec<&str>, options: &ParserOptions) -> MoleculeData {
    let model_count = count_models(&lines);

    // 多个分子但用户没开启
    if model_count > 0 && !options.multimodel {
        panic!(
            "Found multiple molecules but 'multimodel' is false. Please enable 'multimodel = true' to parse all molecules."
        );
    }

    // 用户开启了但其实只有一个
    if model_count == 0 && options.multimodel {
        panic!(
            "Only one molecule found, but 'multimodel = true' was set. Consider setting 'multimodel = false' to avoid confusion."
        );
    }

    let mut molecules = vec![vec![]];
    let mut current = 0;

    while lines.len() >= 8 {
        if !lines[4].starts_with("M  V30 BEGIN CTAB") || !lines[5].starts_with("M  V30 COUNTS") {
            break;
        }

        let counts: Vec<_> = lines[5][13..].split_whitespace().collect();
        let atom_count = counts
            .get(0)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let bond_count = counts
            .get(1)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let mut offset = 7;

        let mut serial_to_index = vec![None; atom_count];
        let start = molecules[current].len();

        for i in 0..atom_count {
            let line = lines[offset + i];
            let parts: Vec<_> = line[6..].split_whitespace().collect();
            if parts.len() > 4 {
                let elem_cap = capitalize(parts[1]);
                if elem_cap != "H" || options.keep_h {
                    let atom = Atom {
                        atom: elem_cap.clone(),
                        elem: elem_cap,
                        x: parts[2].parse().unwrap_or(0.0),
                        y: parts[3].parse().unwrap_or(0.0),
                        z: parts[4].parse().unwrap_or(0.0),
                        serial: start + i,
                        index: molecules[current].len(),
                        hetflag: true,
                        bonds: vec![],
                        bond_order: vec![],
                        properties: std::collections::HashMap::new(),
                    };
                    serial_to_index[i] = Some(molecules[current].len());
                    molecules[current].push(atom);
                }
            }
        }

        offset += atom_count + 1; // skip "END ATOM"
        offset += 1; // BEGIN BOND

        for i in 0..bond_count {
            let line = lines[offset + i];
            let parts: Vec<_> = line[6..].split_whitespace().collect();
            if parts.len() > 3 {
                let from = parts[2].parse::<usize>().unwrap_or(0).saturating_sub(1);
                let to = parts[3].parse::<usize>().unwrap_or(0).saturating_sub(1);
                let order = parts[1].parse::<f32>().unwrap_or(1.0);
                if let (Some(f), Some(t)) = (
                    serial_to_index.get(from).and_then(|x| *x),
                    serial_to_index.get(to).and_then(|x| *x),
                ) {
                    molecules[current][f].bonds.push(t);
                    molecules[current][f].bond_order.push(order);
                    molecules[current][t].bonds.push(f);
                    molecules[current][t].bond_order.push(order);
                }
            }
        }

        let mut next_offset = offset + bond_count;
        if options.multimodel {
            if !options.onemol {
                molecules.push(vec![]);
                current += 1;
            }
            while next_offset < lines.len() && lines[next_offset] != "$$$$" {
                next_offset += 1;
            }
            lines.drain(0..=next_offset);
        } else {
            break;
        }
    }

    molecules
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => {
            first.to_ascii_uppercase().to_string() + &chars.as_str().to_ascii_lowercase()
        }
        None => String::new(),
    }
}

fn count_models(lines: &[&str]) -> usize {
    lines.iter().filter(|line| line.trim() == "$$$$").count()
}
