//! A Japanese morphological analyzer written in pure Rust.
//!
//! ## Examples
//!
//! ```
//! use yoin::ipadic;
//!
//! let input = "すもももももももものうち";
//!
//! let tokenizer = ipadic::tokenizer();
//!
//! for token in tokenizer.tokenize(input) {
//!     println!("{}", token.surface());
//!     println!("{}", &input[token.start()..token.end()]);
//!     for feature in token.features() {
//!         println!("{}", feature);
//!     }
//! }
//! ```
extern crate byteorder;

pub mod dic;
pub mod sysdic;
#[cfg(feature = "ipadic")]
pub mod ipadic;
pub mod tokenizer;

pub const VERSION: &'static str = "0.0.1";
