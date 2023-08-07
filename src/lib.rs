//! A crate for decoding Nintendo Jupiter Texture files, integrated with the
//! `image` crate.
//!
//! # Usage
//! ```rust,no_run
//! # use std::{error::Error, fs::File};
//! #
//! # use image::{DynamicImage, ImageFormat};
//! # use jtex::JupiterDecoder;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! #
//! let file = File::open("Prs_P_254_C.jtex")?;
//! let decoder = JupiterDecoder::decode(file)?;
//! let image = DynamicImage::from_decoder(decoder)?;
//!
//! image.save_with_format("Prs_P_254_C.png", ImageFormat::Png)?;
//! #
//! # Ok(())
//! # }
//! ```

// Linting Rules
#![warn(
	clippy::complexity,
	clippy::correctness,
	clippy::pedantic,
	clippy::perf,
	clippy::style,
	clippy::suspicious,
	clippy::clone_on_ref_ptr,
	clippy::dbg_macro,
	clippy::decimal_literal_representation,
	clippy::exhaustive_enums,
	clippy::exhaustive_structs,
	clippy::filetype_is_file,
	clippy::if_then_some_else_none,
	clippy::non_ascii_literal,
	clippy::self_named_module_files,
	clippy::str_to_string,
	clippy::undocumented_unsafe_blocks,
	clippy::use_debug,
	clippy::wildcard_enum_match_arm,
	missing_docs,
	rustdoc::missing_crate_level_docs
)]
#![allow(
	clippy::cast_possible_truncation,
	clippy::cast_possible_wrap,
	clippy::cast_precision_loss,
	clippy::cast_sign_loss,
	clippy::doc_markdown,
	clippy::module_name_repetitions,
	clippy::similar_names,
	clippy::too_many_lines,
	clippy::unnecessary_wraps,
	dead_code,
	unused_macros
)]

// Modules
mod colour_type;
mod decoder;
mod error;
mod util;

// Exports
pub use self::{decoder::*, error::*};
