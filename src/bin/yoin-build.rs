extern crate encoding;
extern crate clap;
extern crate byteorder;

use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::convert::From;
use std::collections::HashMap;

use clap::{App, Arg};
use encoding::{Encoding, DecoderTrap};
use encoding::all::EUC_JP;

extern crate yoin;

use yoin::dict::fst::Fst;
use yoin::dict::{Morph, Matrix};
use yoin::dict::unknown::{CategoryId, Category, CharTable, UnkDict, Entry};

#[derive(Debug)]
enum Error {
    InvalidMorph,
    IO(io::Error),
    InvalidEncode,
    InvalidMatrix,
    InvalidChardef,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

fn read_csv<P: AsRef<Path>>(buf: &mut Vec<String>, path: P) -> Result<(), Error> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    let contents = EUC_JP.decode(&contents, DecoderTrap::Strict).map_err(|_| Error::InvalidEncode)?;
    for line in contents.lines() {
        buf.push(line.to_string());
    }
    Ok(())
}

fn build_morph(s: &str) -> Result<Morph<&str>, Error> {
    let tokens = s.splitn(5, ',').collect::<Vec<_>>();
    if tokens.len() != 5 {
        return Err(Error::InvalidMorph);
    }
    let surface = tokens[0];
    let left_id = tokens[1].parse::<u16>().map_err(|_| Error::InvalidMorph)?;
    let right_id = tokens[2].parse::<u16>().map_err(|_| Error::InvalidMorph)?;
    let weight = tokens[3].parse::<i16>().map_err(|_| Error::InvalidMorph)?;
    let contents = tokens[4];
    Ok(Morph {
        surface: surface,
        left_id: left_id,
        right_id: right_id,
        weight: weight,
        contents: contents,
    })
}

fn build_entries(morphs: &[String]) -> Result<(Vec<(&[u8], u32)>, Vec<u8>), Error> {
    let morphs = morphs.iter().map(|s| build_morph(s));
    let mut inputs = Vec::new();
    let mut bytes = Vec::new();
    for morph in morphs {
        let morph = morph?;
        let index = bytes.len();
        inputs.push((morph.surface.as_bytes(), index as u32));
        morph.encode_native(&mut bytes)?;
    }
    Ok((inputs, bytes))
}

fn read_matrix<P: AsRef<Path>>(path: P) -> Result<Matrix<Vec<i16>>, Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    let width_height = first_line.trim().split(" ").collect::<Vec<_>>();
    if width_height.len() != 2 {
        return Err(Error::InvalidMatrix);
    }
    let width = width_height[0].parse::<u16>().map_err(|_| Error::InvalidMatrix)?;
    let height = width_height[1].parse::<u16>().map_err(|_| Error::InvalidMatrix)?;
    let mut matrix = Matrix::with_zeros(width, height);
    for line in reader.lines() {
        let line = line?;
        let tokens = line.split(" ").collect::<Vec<_>>();
        if tokens.len() != 3 {
            return Err(Error::InvalidMatrix);
        }
        let w = tokens[0].parse::<u16>().map_err(|_| Error::InvalidMatrix)?;
        let h = tokens[1].parse::<u16>().map_err(|_| Error::InvalidMatrix)?;
        let cost = tokens[2].parse::<i16>().map_err(|_| Error::InvalidMatrix)?;
        matrix[(w, h)] = cost;
    }
    Ok(matrix)
}

fn build_chardef(contents: &str) -> Result<(CharTable, HashMap<String, CategoryId>), Error> {
    let lines =
        contents.lines().filter(|s| !s.is_empty() && !s.starts_with('#')).collect::<Vec<_>>();
    let mut table = HashMap::new();
    let mut cates = Vec::new();
    let mut default = None;

    let mut i = 0;
    while !lines[i].starts_with("0x") {
        let cols = lines[i]
            .trim()
            .split(|c: char| c == '\t' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        i += 1;
        if cols.len() < 4 {
            continue;
        }
        let name = cols[0];
        let invoke = cols[1] == "1";
        let group = cols[2] == "1";
        let length = cols[3].parse::<u8>().map_err(|_| Error::InvalidChardef)?;
        let cate = Category {
            invoke: invoke,
            group: group,
            length: length,
        };
        let id = cates.len() as u8;
        table.insert(name.to_string(), id);
        cates.push(cate);
        if name == "DEFAULT" {
            default = Some(id);
        }
    }
    let mut char_table = match default {
        None => return Err(Error::InvalidChardef),
        Some(id) => CharTable::new(id, cates),
    };

    for line in &lines[i..] {
        let cols = line.trim()
            .split(|c: char| c == '\t' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        // character range...
        if cols.len() < 2 {
            continue;
        }
        let range = cols[0].split("..").collect::<Vec<_>>();
        let cate = cols[1];
        let cate_id = table[cate];
        if range.len() == 1 {
            let c = u32::from_str_radix(&range[0][2..], 16).map_err(|_| Error::InvalidChardef)?;
            char_table.set(c as usize, cate_id);
        } else {
            let start = u32::from_str_radix(&range[0][2..], 16).map_err(|_| Error::InvalidChardef)?;
            let end = u32::from_str_radix(&range[1][2..], 16).map_err(|_| Error::InvalidChardef)?;
            for c in start..end {
                char_table.set(c as usize, cate_id);
            }
        }
    }

    Ok((char_table, table))
}

fn build_unknown_dic<P: AsRef<Path>>(dicdir: P) -> Result<UnkDict, Error> {
    let (char_table, cate_table) = {
        let mut buf = Vec::new();
        File::open(dicdir.as_ref().join("char.def"))?.read_to_end(&mut buf)?;
        let contents = EUC_JP.decode(&buf, DecoderTrap::Strict)
            .map_err(|_| Error::InvalidEncode)?;
        build_chardef(&contents)?
    };

    let mut contents = Vec::new();
    File::open(dicdir.as_ref().join("unk.def"))?.read_to_end(&mut contents)?;
    let contents = EUC_JP.decode(&contents, DecoderTrap::Strict).map_err(|_| Error::InvalidEncode)?;
    let mut entries = HashMap::new();
    for line in contents.lines() {
        let cols = line.trim().splitn(5, ',').collect::<Vec<_>>();
        if cols.len() != 5 {
            return Err(Error::InvalidMorph);
        }
        let cate = cols[0];
        let left_id = cols[1].parse::<u16>().map_err(|_| Error::InvalidMorph)?;
        let right_id = cols[2].parse::<u16>().map_err(|_| Error::InvalidMorph)?;
        let weight = cols[3].parse::<i16>().map_err(|_| Error::InvalidMorph)?;
        let contents = cols[4];
        let entry = Entry {
            left_id: left_id,
            right_id: right_id,
            weight: weight,
            contents: contents,
        };
        let id = cate_table[cate];
        entries.entry(id).or_insert(Vec::new()).push(entry);
    }
    Ok(UnkDict::build(entries, char_table))
}

fn build() -> Result<(), Error> {
    let matches = App::new("yoin-build")
        .version("0.0.1")
        .arg(Arg::with_name("dict")
            .value_name("DIR")
            .help("directory that contains dictionaries")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("outdir")
            .value_name("OUTDIR")
            .help("output directory")
            .takes_value(true))
        .get_matches();
    let dict = match matches.value_of("dict") {
        Some(dict) => Path::new(dict),
        None => unreachable!(),
    };
    let outdir_name = match matches.value_of("outdir") {
        Some(dir) => dir,
        None => ".",
    };
    let outdir = Path::new(outdir_name);
    if !outdir.is_dir() {
        fs::create_dir_all(&outdir)?;
    }
    let mut morphs = Vec::new();
    println!("Reading csv files...");
    for entry in fs::read_dir(&dict)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "csv" {
                read_csv(&mut morphs, &path)?;
            }
        }
    }
    let (mut inputs, entries) = build_entries(&morphs)?;
    println!("sort...");
    inputs.sort();
    println!("building MAST and bytecode");
    let f = Fst::build(inputs);
    println!("dumping...");
    let dic_path = outdir.join("ipadic.dic");
    let mut out = File::create(dic_path)?;
    out.write_all(f.bytecode())?;
    let entries_path = outdir.join("ipadic.morph");
    let mut out = File::create(entries_path)?;
    out.write_all(&entries)?;
    println!("reading matrix...");
    let matrix = read_matrix(dict.join("matrix.def"))?;
    println!("dumping...");
    let mut out = File::create(outdir.join("ipadic.matrix"))?;
    matrix.encode_native(&mut out)?;
    println!("reading char.def and unk.def");
    let unkdic = build_unknown_dic(&dict)?;
    println!("dumping...");
    let out = File::create(outdir.join("ipadic.unk"))?;
    unkdic.encode_native(out)?;
    Ok(())
}

fn main() {
    build().unwrap();
}
