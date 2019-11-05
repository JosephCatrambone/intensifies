
use base64;
use std::fs::File;
use std::io::{Read, Write};

use crate::image_processing::generate;

pub fn run_as_cli(input_filename: &String, output_filename: &String, text: &String) {
	// TEST!  This is wrapping as B64 instead of passing bytes directly because we are testing!
	let mut input = File::open(input_filename).unwrap();
	let mut data = Vec::<u8>::new();
	input.read_to_end(&mut data);
	let intense = generate(&base64::encode(&data), text, [255, 0, 255, 255], 2, 3);
	match intense {
		Ok(res) => {
			let decoded = base64::decode(&res).unwrap();
			let mut output = File::create(&output_filename).unwrap();
			output.write_all(decoded.as_slice());
		},
		Err(err) => {
			println!("Too intense:");
			println!("{}", err);
		}
	}
}
