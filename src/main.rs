
mod cli;
mod image_processing;
mod web;

use cli::run_as_cli;
use std::env;
use web::start_web_service;

fn main() {
	let args: Vec<String> = env::args().collect();
	
	if args[1].eq_ignore_ascii_case("web") {
		start_web_service();
	} else {
		let input_filename = &args[1];
		let output_filename = &args[2];
		let text = &args[3];
		run_as_cli(input_filename, output_filename, text);
	}
}
