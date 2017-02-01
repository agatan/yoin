extern crate bzip2;
extern crate tar;

use std::fs::File;
use std::path::Path;

use bzip2::read::BzDecoder;
use tar::Archive;

fn main() {
    let data_dir = Path::new("./data");
    if data_dir.is_dir() {
        return;
    }

    let compressed = File::open("./data.tar.bz2").unwrap();
    let decoder = BzDecoder::new(compressed);
    let mut archive = Archive::new(decoder);

    for file in archive.entries().unwrap() {
        let mut file = file.unwrap();

        file.unpack_in(".").unwrap();
    }
}
