use std::env::args;

use image::{ImageBuffer, Rgba};
use jtex::jupiter::Jupiter;

pub fn main() -> Result<(), ()> {
	let args = args().collect::<Vec<_>>();
	if args.len() < 2 {
		return Err(());
	}

	// "/mnt/Emus/Tools/DotNet 3DS Toolkit/RawFiles/RomFS/Prs_P_254_C.jtex"
	let j = Jupiter::open(args[1].as_str());

	// dbg!(&j);
	match j {
		Ok(jupiter) => {
			let mut image_buffer = ImageBuffer::new(jupiter.get_width(), jupiter.get_height());

			let image_data = jupiter.get_image_data();
			for i in 0..(jupiter.get_width() * jupiter.get_height()) {
				let x = i % jupiter.get_width();
				let y = i / jupiter.get_width();

				let pixel = &image_data[i as usize];
				image_buffer.put_pixel(
					x,
					y,
					Rgba([pixel.red, pixel.green, pixel.blue, pixel.alpha]),
				);
			}

			image_buffer.save("test.png").unwrap();
		}
		Err(error) => {
			eprintln!("{error}");
			return Err(());
		}
	}

	Ok(())
}
