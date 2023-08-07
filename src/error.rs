// Uses
use std::{error::Error as StdError, io::Error as IoError};

use nintendo_lz::errors::{InvalidMagicNumberError, OutOfRangeError};
use thiserror::Error;

/// The crate error type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
	/// The decoder encountered a Jupiter colour type that it does not
	/// recognise.
	#[error("unknown colour type: {0}")]
	ColourType(u32),
	/// An I/O error occurred.
	#[error("I/O error: {0}")]
	Io(#[from] IoError),
	/// An error occurred during (de)compression.
	#[error("(de)compression error: {0}")]
	Compression(#[from] NintendoLzError),
}

/// A wrapper error type for errors that come from `nintendo-lz`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum NintendoLzError {
	/// The magic number for the format is invalid.
	#[error(transparent)]
	InvalidMagicNumber(InvalidMagicNumberError),
	/// A configured parameter is out of range.
	#[error(transparent)]
	OutOfRange(OutOfRangeError),
	/// Any other error that may come from `nintendo-lz`, including standard
	/// errors.
	#[error("unknown `nintendo-lz` error: {0}")]
	Unknown(Box<dyn StdError>),
}

impl From<Box<dyn StdError>> for NintendoLzError {
	fn from(error: Box<dyn StdError>) -> Self {
		if let Some(e) = error.downcast_ref::<InvalidMagicNumberError>() {
			NintendoLzError::InvalidMagicNumber(e.clone())
		} else if let Some(e) = error.downcast_ref::<OutOfRangeError>() {
			NintendoLzError::OutOfRange(e.clone())
		} else {
			NintendoLzError::Unknown(error)
		}
	}
}
