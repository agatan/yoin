extern crate encoding;
extern crate clap;
extern crate byteorder;

use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;

use clap::{App, Arg};
use encoding::{Encoding, DecoderTrap};
use encoding::all::EUC_JP;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

extern crate yoin;

use yoin::mast;
use yoin::ir;
use yoin::op;
use yoin::dict;

fn try_fst() {
    let samples: Vec<(&[u8], [u8; 4])> = vec![(b"apr", [0, 0, 3, 0]),
                                              (b"aug", [0, 0, 3, 1]),
                                              (b"dec", [0, 0, 3, 1]),
                                              (b"feb", [0, 0, 2, 8]),
                                              (b"feb", [0, 0, 2, 9]),
                                              (b"jan", [0, 0, 3, 1]),
                                              (b"jul", [0, 0, 3, 0]),
                                              (b"jun", [0, 0, 3, 1])];
    let samples = samples.into_iter()
        .map(|(x, bytes)| {
            let out: i32 = unsafe { ::std::mem::transmute(bytes) };
            (x, out)
        });
    let m = mast::Mast::build(samples);

    println!("build MAST and interpret");
    for out in m.run(b"feb").unwrap() {
        let buf: [u8; 4] = unsafe { ::std::mem::transmute(out) };
        println!("{:?}", buf);
    }

    println!("build IR and interpret");
    for out in ir::run(&m, b"feba").unwrap() {
        let (n, substr) = out;
        let buf: [u8; 4] = unsafe { ::std::mem::transmute(n) };
        println!("{}: {:?}", String::from_utf8_lossy(substr), buf);
    }

    println!("bytecode interpret");
    let bytecode = op::build(m);
    for out in op::run_iter(&bytecode, b"feba") {
        let out = out.unwrap();
        let buf: [u8; 4] = unsafe { ::std::mem::transmute(out.value) };
        println!("{}: {:?}",
                 String::from_utf8_lossy(&b"feba"[..out.len]),
                 buf);
    }
}

fn read_csv<P: AsRef<Path>>(buf: &mut Vec<String>, path: P) {
    let mut file = File::open(path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let contents = EUC_JP.decode(&contents, DecoderTrap::Strict).unwrap();
    for line in contents.lines() {
        buf.push(line.to_string());
    }
}

fn build_entries(entries: &[String]) -> (Vec<(&[u8], i32)>, Vec<u8>) {
    let mut inputs = Vec::new();
    let mut bytes = Vec::new();
    for entry in entries {
        let index = bytes.len();
        let token = entry.as_str().split(',').next().unwrap();
        inputs.push((token.as_bytes(), index as i32));
        let size = entry.len() as u32;
        bytes.write_u32::<LittleEndian>(size).unwrap();
        bytes.write_all(entry.as_bytes()).unwrap();
    }
    (inputs, bytes)
}

fn build() {
    let matches = App::new("yoin")
        .version("0.0.1")
        .arg(Arg::with_name("dict").value_name("DIR").takes_value(true))
        .arg(Arg::with_name("output").value_name("FILE").takes_value(true))
        .get_matches();
    let dict = match matches.value_of("dict") {
        Some(dict) => dict,
        None => {
            try_fst();
            return;
        }
    };
    let out = matches.value_of("output").unwrap();

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
    println!("building MAST");
    let m = mast::Mast::build(inputs);
    println!("building byte code");
    let bytecodes = op::build(m);
    println!("dumping...");
    let mut out = File::create("mecab.dic").unwrap();
    out.write_all(&bytecodes).unwrap();
    let mut out = File::create("mecab.entries").unwrap();
    out.write_all(&entries).unwrap();
}

fn run() {
    let bytecodes = include_bytes!("../mecab.dic");
    let entries = include_bytes!("../mecab.entries");
    let dict = dict::Dict::from_bytes(bytecodes, entries);
    let input = env::args().nth(1).unwrap();
    let morphs = dict.run(input.as_bytes()).unwrap();
    for morph in &morphs {
        println!("{}", morph);
    }
}

fn main() {
    // build();
    run();
}
