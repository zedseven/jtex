/// Debugging function for printing bytes to stdout.
pub fn print_bytes(bytes: &[u8]) {
	const BYTES_PER_ROW: usize = 16;

	for line in bytes.chunks(BYTES_PER_ROW) {
		// Print the hex
		let mut first = true;
		for byte in line {
			if first {
				first = false;
			} else {
				print!(" ");
			}
			print!("{:0>2X}", byte);
		}

		// End the line
		println!();
	}
}

/// Rounds up to the next multiple of the base.
pub fn next_multiple_of(base: u32, num: u32) -> u32 {
	((num + base - 1) / base) * base
}

/// Rounds up to the next power of two.
pub fn next_largest_power_of_2(mut num: u32) -> u32 {
	num -= 1;
	num |= num >> 1;
	num |= num >> 2;
	num |= num >> 4;
	num |= num >> 8;
	num |= num >> 16;
	num + 1
}

pub fn decimal_ordinate_to_x_y(ordinate: u32) -> (u32, u32) {
	let mut x = ordinate;
	let mut y = ordinate >> 1;
	x &= 0b01010101010101010101010101010101;
	y &= 0b01010101010101010101010101010101;
	x |= x >> 1;
	y |= y >> 1;
	x &= 0b00110011001100110011001100110011;
	y &= 0b00110011001100110011001100110011;
	x |= x >> 2;
	y |= y >> 2;
	x &= 0b00001111000011110000111100001111;
	y &= 0b00001111000011110000111100001111;
	x |= x >> 4;
	y |= y >> 4;
	x &= 0b00000000111111110000000011111111;
	y &= 0b00000000111111110000000011111111;
	x |= x >> 8;
	y |= y >> 8;
	x &= 0b00000000000000001111111111111111;
	y &= 0b00000000000000001111111111111111;

	(x, y)
}
