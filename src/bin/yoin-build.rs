extern crate encoding;
extern crate clap;
extern crate byteorder;

use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::convert::From;

use clap::{App, Arg};
use encoding::{Encoding, DecoderTrap};
use encoding::all::EUC_JP;

extern crate yoin;

use yoin::dict::fst::Fst;
use yoin::dict::{Morph, Matrix};

#[derive(Debug)]
enum Error {
    InvalidMorph,
    IO(io::Error),
    InvalidEncode,
    InvalidMatrix,
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

fn build() -> Result<(), Error>{
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
        None => {
            unreachable!()
        }
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
    Ok(())
}

fn main() {
    build().unwrap();
}
