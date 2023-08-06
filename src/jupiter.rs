// Uses
use std::{fs::File, io::Read, path::Path};

use nintendo_lz::decompress_arr;

use crate::{
	error::{Error, NintendoLzError},
	util::{decimal_ordinate_to_x_y, next_largest_power_of_2, next_multiple_of, print_bytes},
};

// Constants
const DATA_TYPE_COMPRESSED: u8 = 0x11;
const TILE_SIZE: u32 = 8;
const TILE_AREA: u32 = TILE_SIZE * TILE_SIZE;

#[derive(Clone, Copy, Debug)]
pub enum ColourType {
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

impl ColourType {
	pub fn bits_per_pixel(self) -> usize {
		match self {
			Self::L8 => 8,
			Self::Rgba4444 => 16,
			Self::Rgba5551 => 16,
			Self::Rgb888 => 24,
			Self::Rgba8888 => 32,
		}
	}

	pub fn bytes_per_pixel(self) -> usize {
		const BITS_PER_BYTE: usize = 8;

		self.bits_per_pixel() / BITS_PER_BYTE
	}
}

impl TryFrom<u32> for ColourType {
	type Error = Error;

	fn try_from(raw_format: u32) -> Result<Self, Self::Error> {
		match raw_format {
			0 => Ok(ColourType::L8),
			2 => Ok(ColourType::Rgba8888),
			3 => Ok(ColourType::Rgb888),
			4 => Ok(ColourType::Rgba4444),
			// 6 is conjecture
			5 | 6 => Ok(ColourType::Rgba5551),
			_ => Err(Error::Format(format!(
				"unknown colour format: {raw_format}"
			))),
		}
	}
}

/// RGBA8888 Pixel.
#[derive(Clone, Debug, Default)]
pub struct Pixel {
	pub red:   u8,
	pub green: u8,
	pub blue:  u8,
	pub alpha: u8,
}

impl Pixel {
	pub fn from_byte_chunk(chunk: &[u8], colour_type: ColourType) -> Self {
		/// Calculated as (2^8 - 1) / (2^1 - 1)
		const BIT_TO_BYTE_MULTIPLIER: u8 = 255;
		/// Calculated as (2^8 - 1) / (2^4 - 1)
		const NIBBLE_TO_BYTE_MULTIPLIER: u8 = 255 / 15;
		/// Calculated by multiplying every possible value in a 0..2^5 (0..32)
		/// range by (2^8 - 1) / (2^5 - 1), then rounding to the nearest integer
		const PENTAD_TO_BYTE_TABLE: [u8; 32] = [
			0x00, 0x08, 0x10, 0x19, 0x21, 0x29, 0x31, 0x3A, 0x42, 0x4A, 0x52, 0x5A, 0x63, 0x6B,
			0x73, 0x7B, 0x84, 0x8C, 0x94, 0x9C, 0xA5, 0xAD, 0xB5, 0xBD, 0xC5, 0xCE, 0xD6, 0xDE,
			0xE6, 0xEF, 0xF7, 0xFF,
		];

		assert_eq!(
			chunk.len(),
			colour_type.bytes_per_pixel(),
			"byte chunk size should match the bytes per pixel count of the colour type"
		);

		match colour_type {
			ColourType::L8 => Self {
				red:   chunk[0],
				green: chunk[0],
				blue:  chunk[0],
				alpha: 0xFF,
			},
			ColourType::Rgba4444 => Self {
				red:   ((chunk[1] >> 4) & 0b00001111) * NIBBLE_TO_BYTE_MULTIPLIER,
				green: (chunk[1] & 0b00001111) * NIBBLE_TO_BYTE_MULTIPLIER,
				blue:  ((chunk[0] >> 4) & 0b00001111) * NIBBLE_TO_BYTE_MULTIPLIER,
				alpha: (chunk[0] & 0b00001111) * NIBBLE_TO_BYTE_MULTIPLIER,
			},
			ColourType::Rgba5551 => {
				let complete_value = u16::from_le_bytes(chunk.try_into().unwrap());

				Self {
					red:   PENTAD_TO_BYTE_TABLE
						[((complete_value & 0b1111100000000000) >> 11) as usize],
					green: PENTAD_TO_BYTE_TABLE
						[((complete_value & 0b0000011111000000) >> 6) as usize],
					blue:  PENTAD_TO_BYTE_TABLE
						[((complete_value & 0b0000000000111110) >> 1) as usize],
					alpha: (complete_value & 0b0000000000000001) as u8 * BIT_TO_BYTE_MULTIPLIER,
				}
			}
			ColourType::Rgb888 => Self {
				red:   chunk[2],
				green: chunk[1],
				blue:  chunk[0],
				alpha: 0xFF,
			},

			ColourType::Rgba8888 => Self {
				red:   chunk[3],
				green: chunk[2],
				blue:  chunk[1],
				alpha: chunk[0],
			},
		}
	}
}

#[derive(Debug)]
pub struct Jupiter {
	width:      u32,
	height:     u32,
	image_data: Vec<Pixel>,
}

impl Jupiter {
	pub fn new(raw_bytes: &[u8]) -> Result<Self, Error> {
		let mut decompressed_data = None;
		let is_compressed = raw_bytes[0] == DATA_TYPE_COMPRESSED;

		let all_bytes = if is_compressed {
			decompressed_data = Some(decompress_arr(raw_bytes).map_err(NintendoLzError::from)?);

			decompressed_data.as_ref().unwrap().as_slice()
		} else {
			raw_bytes
		};

		print_bytes(raw_bytes);
		println!();
		print_bytes(all_bytes);
		println!();

		let header_length = u32::from_le_bytes(all_bytes[0x0..0x4].try_into().unwrap());
		let colour_type =
			ColourType::try_from(u32::from_le_bytes(all_bytes[0x4..0x8].try_into().unwrap()))?;
		let width = u32::from_le_bytes(all_bytes[0x8..0xC].try_into().unwrap());
		let height = u32::from_le_bytes(all_bytes[0xC..0x10].try_into().unwrap());

		dbg!(header_length);
		dbg!(colour_type);
		dbg!(width);
		dbg!(height);

		let image_bytes = &all_bytes[(header_length as usize)..];

		let new_width = next_largest_power_of_2(next_multiple_of(TILE_SIZE, width));
		let new_height = next_largest_power_of_2(next_multiple_of(TILE_SIZE, height));

		dbg!(new_width);
		dbg!(new_height);

		let new_area = new_width * new_height;
		if new_area != (image_bytes.len() / colour_type.bytes_per_pixel()) as u32 {
			return Err(Error::Format("image data length mismatch".to_owned()));
		}

		let mut image_data = vec![Pixel::default(); new_area as usize];
		let tiles_per_row = {
			let x = new_width / TILE_SIZE;
			if x == 0 {
				1
			} else {
				x
			}
		};

		for (input_pixel_index, image_byte_chunk) in image_bytes
			.chunks(colour_type.bytes_per_pixel())
			.enumerate()
		{
			let tile_index = input_pixel_index as u32 / TILE_AREA;
			let tile_x = tile_index % tiles_per_row;
			let tile_y = tile_index / tiles_per_row;
			// let pixel_x = tile_x * TILE_SIZE + (input_pixel_index as u32 % TILE_AREA) %
			// TILE_SIZE; let pixel_y = tile_y * TILE_SIZE + (input_pixel_index as u32 %
			// TILE_AREA) / TILE_SIZE;
			let (mut pixel_x, mut pixel_y) =
				decimal_ordinate_to_x_y(input_pixel_index as u32 % TILE_AREA);
			pixel_x += tile_x * TILE_SIZE;
			pixel_y += tile_y * TILE_SIZE;

			let output_pixel_index = pixel_y * new_width + pixel_x;

			// dbg!(input_pixel_index);
			// dbg!(tile_index);
			// dbg!(tile_x);
			// dbg!(tile_y);
			// dbg!(pixel_x);
			// dbg!(pixel_y);
			// dbg!(output_pixel_index);
			// dbg!(image_byte_chunk);

			image_data[output_pixel_index as usize] =
				Pixel::from_byte_chunk(image_byte_chunk, colour_type);
		}

		Ok(Self {
			width: new_width,
			height: new_height,
			image_data,
		})
	}

	pub fn open<P>(path: P) -> Result<Self, Error>
	where
		P: AsRef<Path>,
	{
		let mut file = File::open(path)?;
		let file_length = file.metadata()?.len() as usize;
		let mut buffer = vec![0; file_length];

		file.read_exact(&mut buffer)?;

		Self::new(&buffer)
	}

	pub fn get_width(&self) -> u32 {
		self.width
	}

	pub fn get_height(&self) -> u32 {
		self.height
	}

	pub fn get_image_data(&self) -> &Vec<Pixel> {
		&self.image_data
	}
}
