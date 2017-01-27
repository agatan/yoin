extern crate encoding;
extern crate clap;
extern crate byteorder;

use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;

use clap::{App, Arg};
use encoding::{Encoding, DecoderTrap};
use encoding::all::EUC_JP;

extern crate yoin;

use yoin::fst::Fst;
use yoin::morph::Morph;

fn read_csv<P: AsRef<Path>>(buf: &mut Vec<String>, path: P) {
    let mut file = File::open(path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let contents = EUC_JP.decode(&contents, DecoderTrap::Strict).unwrap();
    for line in contents.lines() {
        buf.push(line.to_string());
    }
}

fn build_entry(s: &str) -> Morph<&str> {
    let mut tokens = s.splitn(5, ',');
    let surface = tokens.next().unwrap();
    let left_id = tokens.next().unwrap().parse::<i16>().unwrap();
    let right_id = tokens.next().unwrap().parse::<i16>().unwrap();
    let weight = tokens.next().unwrap().parse::<i16>().unwrap();
    let contents = tokens.next().unwrap();
    Morph {
        surface: surface,
        left_id: left_id,
        right_id: right_id,
        weight: weight,
        contents: contents,
    }
}

fn build_entries(entries: &[String]) -> (Vec<(&[u8], i32)>, Vec<u8>) {
    let mut inputs = Vec::new();
    let mut bytes = Vec::new();
    for entry in entries {
        let index = bytes.len();
        let morph = build_entry(&entry);
        inputs.push((morph.surface.as_bytes(), index as i32));
        morph.encode_native(&mut bytes).unwrap();
    }
    (inputs, bytes)
}

fn main() {
    let matches = App::new("yoin-build")
        .version("0.0.1")
        .arg(Arg::with_name("dict")
            .value_name("DIR")
            .help("directory that contains dictionaries")
            .takes_value(true))
        .arg(Arg::with_name("outdir")
            .value_name("OUTDIR")
            .help("output directory")
            .takes_value(true))
        .get_matches();
    let dict = match matches.value_of("dict") {
        Some(dict) => dict,
        None => {
            return;
        }
    };
    let outdir_name = match matches.value_of("outdir") {
        Some(dir) => dir,
        None => ".",
    };
    let outdir = Path::new(outdir_name);
    if !outdir.is_dir() {
        fs::create_dir_all(&outdir).unwrap();
    }
    let mut morphs = Vec::new();
    println!("Reading csv files...");
    for entry in fs::read_dir(&dict).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "csv" {
                read_csv(&mut morphs, &path);
            }
        }
    }
    let (mut inputs, entries) = build_entries(&morphs);
    println!("sort...");
    inputs.sort();
    println!("building MAST and bytecode");
    let f = Fst::build(inputs);
    println!("dumping...");
    let dic_path = outdir.join("mecab.dic");
    let mut out = File::create(dic_path).unwrap();
    out.write_all(f.bytecode()).unwrap();
    let entries_path = outdir.join("mecab.entries");
    let mut out = File::create(entries_path).unwrap();
    out.write_all(&entries).unwrap();
}
