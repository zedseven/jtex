// Uses
use std::io::{Error as IoError, ErrorKind, Read};

use byteorder::{ByteOrder, LittleEndian};
use image::{ColorType, ExtendedColorType, ImageDecoder, ImageResult};
use nintendo_lz::decompress_arr;

use crate::{
	error::{Error, NintendoLzError},
	util::{decimal_ordinate_to_x_y, next_largest_power_of_2, next_multiple_of},
};

// Constants
const COMPRESSED_DATA_MARKER_LZ10: u8 = 0x10;
const COMPRESSED_DATA_MARKER_LZ11: u8 = 0x11;
const TILE_SIZE: u32 = 8;
const TILE_AREA: u32 = TILE_SIZE * TILE_SIZE;

// This implementation is a little silly, since it reads the whole image into
// memory and doesn't stream its contents.
// The reason is that this image format doesn't lend itself to streaming - its
// compression algorithm and tiled nature make this unrealistic.
// Fortunately, images of this type are often tiny, so this isn't much of a
// concern.
pub struct JupiterReader {
	width:               u32,
	height:              u32,
	colour_type:         ColorType,
	original_color_type: ExtendedColorType,
	pixel_buffer:        Vec<u8>,
	read_offset:         usize,
}

impl JupiterReader {
	fn open<R>(mut reader: R) -> Result<JupiterReader, Error>
	where
		R: Read,
	{
		const BYTES_PER_U32: usize = 4;

		let mut byte_buffer = Vec::new();
		reader.read_to_end(&mut byte_buffer)?;

		// If the first byte is a compressed data marker, the rest of the image data
		// (including the header) is compressed
		if byte_buffer[0] == COMPRESSED_DATA_MARKER_LZ10
			|| byte_buffer[0] == COMPRESSED_DATA_MARKER_LZ11
		{
			byte_buffer = decompress_arr(byte_buffer.as_slice()).map_err(NintendoLzError::from)?;
		}

		if byte_buffer.len() < BYTES_PER_U32 * 4 {
			return Err(Error::Io(IoError::new(
				ErrorKind::UnexpectedEof,
				"file ended early",
			)));
		}

		let header_length = LittleEndian::read_u32(&byte_buffer[0x00..0x04]);
		let colour_format =
			JupiterColourType::try_from(LittleEndian::read_u32(&byte_buffer[0x04..0x08]))?;
		let width = LittleEndian::read_u32(&byte_buffer[0x08..0x0C]);
		let height = LittleEndian::read_u32(&byte_buffer[0x0C..0x10]);
		let area = width * height;

		let padded_width = next_largest_power_of_2(next_multiple_of(TILE_SIZE, width));
		let padded_height = next_largest_power_of_2(next_multiple_of(TILE_SIZE, height));
		let padded_area = padded_width * padded_height;

		if padded_area
			!= (byte_buffer[(header_length as usize)..].len() / colour_format.bytes_per_pixel())
				as u32
		{
			return Err(Error::Io(IoError::new(
				ErrorKind::UnexpectedEof,
				"file ended early",
			)));
		}

		let original_color_type = ExtendedColorType::from(colour_format);
		let output_colour_type = ColorType::from(colour_format);
		let input_bytes_per_pixel = colour_format.bytes_per_pixel();
		let output_bytes_per_pixel = output_colour_type.bytes_per_pixel() as usize;

		let mut pixel_buffer = vec![0u8; area as usize * output_bytes_per_pixel];

		let tiles_per_row = padded_width / TILE_SIZE;

		for (input_pixel_index, image_byte_chunk) in byte_buffer[(header_length as usize)..]
			.chunks(input_bytes_per_pixel)
			.enumerate()
		{
			let tile_index = input_pixel_index as u32 / TILE_AREA;
			let tile_x = tile_index % tiles_per_row;
			let tile_y = tile_index / tiles_per_row;
			let (mut pixel_x, mut pixel_y) =
				decimal_ordinate_to_x_y(input_pixel_index as u32 % TILE_AREA);
			pixel_x += tile_x * TILE_SIZE;
			pixel_y += tile_y * TILE_SIZE;

			// Crop out the padding pixels
			if pixel_x >= width || pixel_y >= height {
				continue;
			}

			let output_pixel_index = (pixel_y * width + pixel_x) as usize * output_bytes_per_pixel;

			Self::copy_and_convert_pixel_data(
				image_byte_chunk,
				&mut pixel_buffer
					[output_pixel_index..(output_pixel_index + output_bytes_per_pixel)],
				colour_format,
			);
		}

		Ok(Self {
			width,
			height,
			colour_type: output_colour_type,
			original_color_type,
			pixel_buffer,
			read_offset: 0,
		})
	}

	fn copy_and_convert_pixel_data(
		input_chunk: &[u8],
		output_chunk: &mut [u8],
		input_colour_type: JupiterColourType,
	) {
		const MULTIPLIER_1_BIT_TO_8_BIT: u8 = 0xFF;
		/// Calculated by multiplying every possible value in a 0..2^4 (0..16)
		/// range by (2^8 - 1) / (2^4 - 1), then rounding to the nearest integer
		const LOOKUP_TABLE_4_BIT_TO_8_BIT: [u8; 16] = [
			0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
			0xEE, 0xFF,
		];
		/// Calculated by multiplying every possible value in a 0..2^5 (0..32)
		/// range by (2^8 - 1) / (2^5 - 1), then rounding to the nearest integer
		const LOOKUP_TABLE_5_BIT_TO_8_BIT: [u8; 32] = [
			0x00, 0x08, 0x10, 0x19, 0x21, 0x29, 0x31, 0x3A, 0x42, 0x4A, 0x52, 0x5A, 0x63, 0x6B,
			0x73, 0x7B, 0x84, 0x8C, 0x94, 0x9C, 0xA5, 0xAD, 0xB5, 0xBD, 0xC5, 0xCE, 0xD6, 0xDE,
			0xE6, 0xEF, 0xF7, 0xFF,
		];

		let output_colour_type = ColorType::from(input_colour_type);

		assert_eq!(
			input_chunk.len(),
			input_colour_type.bytes_per_pixel(),
			"input chunk size should match the bytes per pixel count of the input colour type"
		);
		assert_eq!(
			output_chunk.len(),
			output_colour_type.bytes_per_pixel() as usize,
			"output chunk size should match the bytes per pixel count of the output colour type"
		);

		match input_colour_type {
			JupiterColourType::L8 => {
				// Both input and output are the same, so a simple copy works here
				output_chunk.copy_from_slice(input_chunk)
			}
			JupiterColourType::Rgba4444 => {
				output_chunk[0] =
					LOOKUP_TABLE_4_BIT_TO_8_BIT[((input_chunk[1] >> 4) & 0b00001111) as usize];
				output_chunk[1] =
					LOOKUP_TABLE_4_BIT_TO_8_BIT[(input_chunk[1] & 0b00001111) as usize];
				output_chunk[2] =
					LOOKUP_TABLE_4_BIT_TO_8_BIT[((input_chunk[0] >> 4) & 0b00001111) as usize];
				output_chunk[3] =
					LOOKUP_TABLE_4_BIT_TO_8_BIT[(input_chunk[0] & 0b00001111) as usize];
			}
			JupiterColourType::Rgba5551 => {
				let complete_value = LittleEndian::read_u16(input_chunk);

				output_chunk[0] = LOOKUP_TABLE_5_BIT_TO_8_BIT
					[((complete_value & 0b1111100000000000) >> 11) as usize];
				output_chunk[1] = LOOKUP_TABLE_5_BIT_TO_8_BIT
					[((complete_value & 0b0000011111000000) >> 6) as usize];
				output_chunk[2] = LOOKUP_TABLE_5_BIT_TO_8_BIT
					[((complete_value & 0b0000000000111110) >> 1) as usize];
				output_chunk[3] =
					(complete_value & 0b0000000000000001) as u8 * MULTIPLIER_1_BIT_TO_8_BIT;
			}
			JupiterColourType::Rgb888 => {
				output_chunk[0] = input_chunk[2];
				output_chunk[1] = input_chunk[1];
				output_chunk[2] = input_chunk[0];
			}
			JupiterColourType::Rgba8888 => {
				output_chunk[0] = input_chunk[3];
				output_chunk[1] = input_chunk[2];
				output_chunk[2] = input_chunk[1];
				output_chunk[3] = input_chunk[0];
			}
		}
	}
}

impl Read for JupiterReader {
	fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IoError> {
		let mut readable_length = self.pixel_buffer.len() - self.read_offset;
		if buffer.len() < readable_length {
			readable_length = buffer.len();
		}

		if readable_length > 0 {
			let mut buffer_slice = &mut buffer[0..readable_length];
			buffer_slice.copy_from_slice(
				&self.pixel_buffer[self.read_offset..(self.read_offset + readable_length)],
			);

			self.read_offset += readable_length;
		}

		Ok(readable_length)
	}
}

pub struct JupiterDecoder {
	reader: JupiterReader,
}

impl JupiterDecoder {
	pub fn new<R>(inner_reader: R) -> Result<JupiterDecoder, Error>
	where
		R: Read,
	{
		Ok(Self {
			reader: JupiterReader::open(inner_reader)?,
		})
	}
}

impl ImageDecoder<'_> for JupiterDecoder {
	type Reader = JupiterReader;

	fn dimensions(&self) -> (u32, u32) {
		(self.reader.width, self.reader.height)
	}

	fn color_type(&self) -> ColorType {
		self.reader.colour_type
	}

	fn original_color_type(&self) -> ExtendedColorType {
		self.reader.original_color_type
	}

	fn into_reader(self) -> ImageResult<Self::Reader> {
		Ok(self.reader)
	}

	fn total_bytes(&self) -> u64 {
		self.reader.pixel_buffer.len() as u64
	}
}

#[derive(Clone, Copy, Debug)]
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
	pub fn bytes_per_pixel(self) -> usize {
		match self {
			Self::L8 => 1,
			Self::Rgba4444 => 2,
			Self::Rgba5551 => 2,
			Self::Rgb888 => 3,
			Self::Rgba8888 => 4,
		}
	}

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
			_ => Err(Error::ColourFormat(raw_format)),
		}
	}
}

impl From<JupiterColourType> for ColorType {
	fn from(value: JupiterColourType) -> Self {
		use JupiterColourType::*;

		match value {
			L8 => Self::L8,
			Rgba4444 => Self::Rgba8,
			Rgba5551 => Self::Rgba8,
			Rgb888 => Self::Rgb8,
			Rgba8888 => Self::Rgba8,
		}
	}
}

impl From<JupiterColourType> for ExtendedColorType {
	fn from(value: JupiterColourType) -> Self {
		use JupiterColourType::*;

		match value {
			L8 => Self::L8,
			Rgba4444 => Self::Rgba4,
			Rgba5551 => Self::Unknown(5),
			Rgb888 => Self::Rgb8,
			Rgba8888 => Self::Rgba8,
		}
	}
}
