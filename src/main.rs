
use base64;
use gif::{Frame, Encoder, Repeat, SetParameter};
use image::{self, imageops, ImageBuffer, DynamicImage, GenericImage, GenericImageView, RgbaImage, Rgba};
use rand;
use rusttype::{point, FontCollection, PositionedGlyph, Scale};
use std::borrow::{Cow, Borrow};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use rand::Rng;
use std::cmp::max;

// Open a base64-encoded image, convert to gif, add [noun intensifies], frames, and export as base64-gif.

fn generate(b64_image: &String, text: &String, font_color: [u8;4], num_frames: u8, shake_intensity: u8) -> Result<String, String> {
	// Decode base64 (standard) into an image vector.
	let image_data = match(base64::decode(b64_image)) {
		Ok(data) => data,
		Err(err) => return Result::Err("Failed to decode b64 image.".to_owned())
	};
	
	let image = match image::load_from_memory(image_data.as_slice()) {
		Ok(image) => image,
		Err(err) => return Result::Err("Failed to decode image in memory.".to_owned())
	};

	// Create a text overlay to put on top of the original image.
	let intensified = generate_image(image, text, font_color, 14.0f32, num_frames, shake_intensity);
	
	// Encode a b64 image.
	let encoded_data = base64::encode(&intensified);
	Ok(encoded_data)
}

fn generate_image(image: DynamicImage, text:&String, font_color: [u8;4], font_size: f32, num_frames: u8, shake_intensity: u8) -> Vec<u8> {
	let shake_intensity = shake_intensity as i8;
	// Generate 'shaking' image.
	// We could use a Vec and read it as bytes because of the bufread support, but this is easer.
	let mut fakefile = Cursor::new(Vec::new());
	let color_map : &[u8] = &[]; // Empty means auto-color.
	{
		let mut encoder = Encoder::new(&mut fakefile, image.width() as u16, image.height() as u16, color_map).unwrap();
		encoder.set(Repeat::Infinite).unwrap();
		
		// Pad the input image so it's easier to avoid OOB errors, then we can randomly crop the center.
		let mut padded_image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(image.width() + (shake_intensity * 2) as u32, image.height() + (shake_intensity * 2) as u32, |x, y| {
			if x < shake_intensity as u32 || y < shake_intensity as u32 || x >= image.width() + shake_intensity as u32 || y >= image.height() + shake_intensity as u32 {
				Rgba([0, 0, 0, 0])
			} else {
				image.get_pixel(x-shake_intensity as u32, y-shake_intensity as u32)
			}
		});
		
		// Add the text.
		overlay_text(&mut padded_image, text, font_color, font_size);
		
		// Create the frames with random crops.
		let mut rng = rand::thread_rng();
		for _ in 0..num_frames {
			// Pick a random offset from 'center' and produce a crop rectangle.
			// Use a gaussian-ish distribution?
			let dx = rng.gen_range(-shake_intensity, shake_intensity);
			let dy = rng.gen_range(-shake_intensity, shake_intensity);
			let subframe = imageops::crop(&mut padded_image, (shake_intensity + dx) as u32, (shake_intensity + dy) as u32, image.width(), image.height()).to_image();
			let mut pixels_raw: Vec<u8> = subframe.into_vec();
			let mut frame = gif::Frame::from_rgba(image.width() as u16, image.height() as u16, pixels_raw.as_mut_slice());
			frame.delay = 2u16;
			encoder.write_frame(&frame);
		}
	}
	
	// Convert the completed file into a b64 image.
	fakefile.seek(SeekFrom::Start(0)).unwrap();
	let mut result_data = Vec::<u8>::new();
	fakefile.read_to_end(&mut result_data);
	
	result_data
}

fn overlay_text(image:&mut ImageBuffer<Rgba<u8>, Vec<u8>>, text:&String, font_color:[u8;4], font_size:f32) {
	// Load font.
	// Generate glyph based on image side.
	let font_data = include_bytes!("../fonts/default.ttf");
	let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap();
	let font = collection
		.into_font() // only succeeds if collection consists of one font
		.unwrap_or_else(|e| {
			panic!("error turning FontCollection into a Font: {}", e);
		});
	
	// Desired font pixel height
	let height: f32 = font_size;
	let width: f32 = font_size;
	
	// 2x scale in x direction to counter the aspect ratio of monospace characters.
	let scale = Scale {
		x: width,
		y: height,
	};
	
	// The origin of a line of text is at the baseline (roughly where
	// non-descending letters sit). We don't want to clip the text, so we shift
	// it down with an offset when laying it out. v_metrics.ascent is the
	// distance between the baseline and the highest edge of any glyph in
	// the font. That's enough to guarantee that there's no clipping.
	let v_metrics = font.v_metrics(scale);
	let font_block_width = width * text.len() as f32;
	let offset = point((image.width() as i32 - font_block_width as i32) as f32/2f32, image.height() as f32 * 2f32 / 3f32 + v_metrics.ascent);
	let glyphs: Vec<PositionedGlyph<'_>> = font.layout(text, scale, offset).collect();
	
	// Find the most visually pleasing width to display
	//let pixel_height = height.ceil() as usize;
	//let width = glyphs.iter().rev().map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width).next().unwrap_or(0.0).ceil() as usize;
	
	// Rasterise directly into ASCII art.
	for g in glyphs {
		if let Some(bb) = g.pixel_bounding_box() {
			g.draw(|x, y, v| {
				// v should be in the range 0.0 to 1.0
				// so something's wrong if you get $ in the output.
				//let i = (v * mapping_scale + 0.5) as usize;
				//let c = mapping.get(i).cloned().unwrap_or(b'$');
				let x = x as i32 + bb.min.x;
				let y = y as i32 + bb.min.y;
				// There's still a possibility that the glyph clips the boundaries of the bitmap
				if x >= 0 && x < image.width() as i32 && y >= 0 && y < image.height() as i32 {
					let x = x as usize;
					let y = y as usize;
					let pixel = image.get_pixel_mut(x as u32, y as u32);
					//pixel_data[(x + y * width)] = c;
					//*pixel[0] = (pixel[0] * 1f32-font_color[3] as f32) + (font_color[0] * font_color[3])
					if v > 0.5 {
						*pixel = image::Rgba(font_color);
					}
				}
			})
		}
	}
}

fn main() {
	// TEST!  This is wrapping as B64 instead of passing bytes directly because we are testing!
	let mut input = File::open("bag.png").unwrap();
	let mut data = Vec::<u8>::new();
	input.read_to_end(&mut data);
	let intense = generate(&base64::encode(&data), &String::from("[Bag Intensifies]"), [255, 0, 255, 255], 2, 3);
	match intense {
		Ok(res) => {
			let decoded = base64::decode(&res).unwrap();
			let mut output = File::create("test.gif").unwrap();
			output.write_all(decoded.as_slice());
		},
		Err(err) => {
			println!("Too intense:");
			println!("{}", err);
		}
	}
}
