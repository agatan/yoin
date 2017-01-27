extern crate encoding;
extern crate clap;
extern crate byteorder;

use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;
use std::io;
use std::convert::From;

use clap::{App, Arg};
use encoding::{Encoding, DecoderTrap};
use encoding::all::EUC_JP;

extern crate yoin;

use yoin::fst::Fst;
use yoin::morph::Morph;

#[derive(Debug)]
enum Error {
    InvalidMorph,
    IO(io::Error),
    InvalidEncode,
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
    let left_id = tokens[1].parse::<i16>().map_err(|_| Error::InvalidMorph)?;
    let right_id = tokens[2].parse::<i16>().map_err(|_| Error::InvalidMorph)?;
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

fn build_entries(morphs: &[String]) -> Result<(Vec<(&[u8], i32)>, Vec<u8>), Error> {
    let morphs = morphs.iter().map(|s| build_morph(s));
    let mut inputs = Vec::new();
    let mut bytes = Vec::new();
    for morph in morphs {
        let morph = morph?;
        let index = bytes.len();
        inputs.push((morph.surface.as_bytes(), index as i32));
        morph.encode_native(&mut bytes)?;
    }
    Ok((inputs, bytes))
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
        Some(dict) => dict,
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
    Ok(())
}

fn main() {
    build().unwrap();
}
