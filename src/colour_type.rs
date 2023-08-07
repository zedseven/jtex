// Uses
use image::{ColorType, ExtendedColorType};

use crate::error::Error;

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum JupiterColourType {
	/// 8 bits per pixel, luminance-only (greyscale)
	L8,
	/// 16 bits per pixel, RGBA (4 bits per channel)
	Rgba4444,
	/// 16 bits per pixel, RGBA (5 bits per colour channel, 1 bit alpha)
	Rgba5551,
	/// 24 bits per pixel, RGB (8 bits per channel)
	Rgb888,
	/// 32 bits per pixel, RGBA (8 bits per channel)
	Rgba8888,
}

impl JupiterColourType {
	#[must_use]
	pub fn bytes_per_pixel(self) -> usize {
		match self {
			Self::L8 => 1,
			Self::Rgba4444 | Self::Rgba5551 => 2,
			Self::Rgb888 => 3,
			Self::Rgba8888 => 4,
		}
	}

	#[must_use]
	pub fn bits_per_pixel(self) -> usize {
		const BITS_PER_BYTE: usize = 8;

		self.bytes_per_pixel() * BITS_PER_BYTE
	}
}

impl TryFrom<u32> for JupiterColourType {
	type Error = Error;

	fn try_from(raw_format: u32) -> Result<Self, Self::Error> {
		match raw_format {
			0 => Ok(JupiterColourType::L8),
			2 => Ok(JupiterColourType::Rgba8888),
			3 => Ok(JupiterColourType::Rgb888),
			4 => Ok(JupiterColourType::Rgba4444),
			// 6 is conjecture
			5 | 6 => Ok(JupiterColourType::Rgba5551),
			_ => Err(Error::ColourType(raw_format)),
		}
	}
}

impl From<JupiterColourType> for ColorType {
	fn from(value: JupiterColourType) -> Self {
		use JupiterColourType::{Rgb888, Rgba4444, Rgba5551, Rgba8888, L8};

		match value {
			L8 => Self::L8,
			Rgb888 => Self::Rgb8,
			Rgba4444 | Rgba5551 | Rgba8888 => Self::Rgba8,
		}
	}
}

impl From<JupiterColourType> for ExtendedColorType {
	fn from(value: JupiterColourType) -> Self {
		use JupiterColourType::{Rgb888, Rgba4444, Rgba5551, Rgba8888, L8};

		match value {
			L8 => Self::L8,
			Rgba4444 => Self::Rgba4,
			Rgba5551 => Self::Unknown(5),
			Rgb888 => Self::Rgb8,
			Rgba8888 => Self::Rgba8,
		}
	}
}
