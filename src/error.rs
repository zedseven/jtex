// Uses
use std::{error::Error as StdError, io::Error as IoError};

use nintendo_lz::errors::{InvalidMagicNumberError, OutOfRangeError};
use thiserror::Error;

/// The crate error type.
#[derive(Debug, Error)]
pub enum Error {
	#[error("format error: {0}")]
	Format(String),
	#[error("io error: {0}")]
	Io(#[from] IoError),
	#[error("(de)compression error: {0}")]
	Compression(#[from] NintendoLzError),
}

#[derive(Debug, Error)]
pub enum NintendoLzError {
	#[error(transparent)]
	InvalidMagicNumber(InvalidMagicNumberError),
	#[error(transparent)]
	OutOfRange(OutOfRangeError),
}

impl From<Box<dyn StdError>> for NintendoLzError {
	fn from(error: Box<dyn StdError>) -> Self {
		if let Some(e) = error.downcast_ref::<InvalidMagicNumberError>() {
			NintendoLzError::InvalidMagicNumber(e.clone())
		} else if let Some(e) = error.downcast_ref::<OutOfRangeError>() {
			NintendoLzError::OutOfRange(e.clone())
		} else {
			panic!("unknown `nintendo-lz` error: {}", error.as_ref());
		}
	}
}