//! A Japanese morphological analyzer written in pure Rust.
//!
//! ## Examples
//!
//! ```
//! use yoin::ipadic;
//!
//! let input = "すもももももももものうち";
//! let expected = vec!["すもも", "も", "もも", "も", "もも", "の", "うち"];
//!
//! let tokenizer = ipadic::tokenizer();
//! let tokens = tokenizer.tokenize(input);
//!
//! for (tok, e) in tokens.iter().zip(expected) {
//!     assert_eq!(tok.surface(), e);
//! }
//! ```
extern crate byteorder;

pub mod dic;
pub mod sysdic;
#[cfg(feature = "ipadic")]
pub mod ipadic;
pub mod tokenizer;

pub const VERSION: &'static str = "0.0.1";
