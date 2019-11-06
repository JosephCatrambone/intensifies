
mod cli;
mod image_processing;
mod web;

use cli::run_as_cli;
use std::env;
use web::start_web_service;

fn main() {
	let args: Vec<String> = env::args().collect();
	let input_filename = &args[1];
	let output_filename = &args[2];
	let text = &args[3];
	
	//run_as_cli(input_filename, output_filename, text);
	start_web_service();
}
