
#[macro_use]
use rouille::try_or_400;
use rouille::input::{json, multipart};
use rouille::Request;
use rouille::Response;
use serde::{Serialize, Deserialize};

use crate::image_processing::generate;

#[derive(Deserialize)]
struct RequestJson {
	image: String,
	text: String,
	shake_frames: u8,
	shake_intensity: u8
	//font_size: f32
}

fn route_handler(request: &Request) -> Response {
	if request.method() == "GET" {
		return Response::html(include_str!("../static/index.html")).with_status_code(200);
	}
	
	let json: RequestJson = try_or_400!(rouille::input::json_input(request));
	
	// Avoid OOM errors.
	assert!(json.shake_frames < 10);
	
	match generate(
			&json.image,
			&json.text,
			json.shake_frames,
			json.shake_intensity
		) {
		Ok(img_text) => Response::text(img_text),
		Err(e) => Response::text(e).with_status_code(500)
	}
	
}

pub fn start_web_service() {
	//rouille::start_server("0.0.0.0:80", move |request| { Response::text("hello world") });
	rouille::start_server("0.0.0.0:80", route_handler);
}
