use std::{env::args, error::Error as ErrorTrait, fs::File};

use image::{DynamicImage, ImageFormat};
use jtex::JupiterDecoder;

pub fn main() -> Result<(), Box<dyn ErrorTrait>> {
	let args = args().collect::<Vec<_>>();
	if args.len() < 2 {
		panic!("not enough arguments supplied");
	}

	// "/mnt/Emus/Tools/DotNet 3DS Toolkit/RawFiles/RomFS/Prs_P_254_C.jtex"
	let file = File::open(args[1].as_str())?;
	let decoder = JupiterDecoder::decode(file)?;
	let image = DynamicImage::from_decoder(decoder)?;
	image.save_with_format("test.png", ImageFormat::Png)?;

	Ok(())
}
