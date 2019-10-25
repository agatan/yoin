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
use yoin_core as core;
pub use yoin_ipadic as ipadic;

pub const VERSION: &'static str = "0.0.1";

pub use crate::core::tokenizer;
