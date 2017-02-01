//! A Japanese morphological analyzer written in pure Rust.
extern crate byteorder;

pub mod dic;
pub mod sysdic;
pub mod tokenizer;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
