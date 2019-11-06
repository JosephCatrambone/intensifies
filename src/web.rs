
#[macro_use]
use rouille::try_or_400;
use rouille::input::{json, multipart};
use rouille::Request;
use rouille::Response;
use serde::{Serialize, Deserialize};

use crate::image_processing::generate;

#[derive(Deserialize)]
struct RequestJson {
	b64_image: String,
	text: String,
	color_r: f32,
	color_g: f32,
	color_b: f32,
	font_size: f32
}

fn route_handler(request: &Request) -> Response {
	let json: RequestJson = try_or_400!(rouille::input::json_input(request));
	
	let img_text = generate(
		&json.b64_image,
		&json.text,
		[(json.color_r * 255u8), (json.color_g * 255u8), (json.color_b * 255u8), 255u8],
		3,
		5
	);
	
	//Response::text(format!("field1's value is {}", json.field1))
	Response::text(img_text)
}

pub fn start_web_service() {
	//rouille::start_server("0.0.0.0:80", move |request| { Response::text("hello world") });
	rouille::start_server("0.0.0.0:80", route_handler);
}
