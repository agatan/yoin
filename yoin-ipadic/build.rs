extern crate bzip2;
extern crate tar;

use std::fs::{self, File};
use std::path::Path;
use std::convert::AsRef;

use bzip2::read::BzDecoder;
use tar::Archive;

fn should_decode_zipped_data<P: AsRef<Path>>(dir: P, zipped: P) -> bool {
    let dir_meta = match fs::metadata(dir) {
        Ok(meta) => meta,
        Err(_) => return true,
    };
    let zipped_meta = match fs::metadata(zipped) {
        Ok(meta) => meta,
        Err(_) => return true,
    };
    if let Ok(dir_time) = dir_meta.modified() {
        if let Ok(zipped_time) = zipped_meta.modified() {
            // the zipped file is newer than the data or not
            return zipped_time > dir_time;
        }
    }
    // can not ensure that the data directory is newer than the zipped.
    return true;
}

fn main() {
    let data_dir = Path::new("./data");
    let zipped = Path::new("./data.tar.bz2");

    if !should_decode_zipped_data(&data_dir, &zipped) {
        return;
    }

    let compressed = File::open(zipped).unwrap();
    let decoder = BzDecoder::new(compressed);
    let mut archive = Archive::new(decoder);

    for file in archive.entries().unwrap() {
        let mut file = file.unwrap();

        file.unpack_in(".").unwrap();
    }
}
