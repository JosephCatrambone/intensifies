use base64;
use gif::{Frame, Encoder, Repeat, SetParameter};
use image::{self, imageops, ImageBuffer, DynamicImage, GenericImage, GenericImageView, RgbaImage, Rgba};
use rand;
use rusttype::{point, FontCollection, PositionedGlyph, Scale};
use std::borrow::{Cow, Borrow};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use rand::Rng;
use std::cmp::{max, min};

// Open a base64-encoded image, convert to gif, add [noun intensifies], frames, and export as base64-gif.

pub fn generate(b64_image: &String, text: &String, num_frames: u8, shake_intensity: u8) -> Result<String, String> {
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
	let intensified = generate_image(image, text, 48.0f32, num_frames, shake_intensity);
	
	// Encode a b64 image.
	let encoded_data = base64::encode(&intensified);
	Ok(encoded_data)
}

pub fn generate_image(image: DynamicImage, text:&String, max_font_size: f32, num_frames: u8, shake_intensity: u8) -> Vec<u8> {
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
		overlay_text(&mut padded_image, text, max_font_size);
		
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

fn overlay_text(image:&mut ImageBuffer<Rgba<u8>, Vec<u8>>, text:&String, max_font_size:f32) {
	// Load font.
	// Generate glyph based on image side.
	let font_data = include_bytes!("../fonts/default.ttf");
	let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap();
	let font = collection
		.into_font() // only succeeds if collection consists of one font
		.unwrap_or_else(|e| {
			panic!("error turning FontCollection into a Font: {}", e);
		});
	
	// 13 pixels is 10 pt.  1.3px -> 1pt.  16 px is 12 pt.  4:3?
	let font_size = f32::min(max_font_size, image.width() as f32 / 1.5f32);
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
	//let offset = point((image.width() as i32 - font_block_width as i32) as f32/2f32, image.height() as f32 * 2f32 / 3f32 + v_metrics.ascent);
	let offset = point(0f32, v_metrics.ascent);
	let glyphs: Vec<PositionedGlyph<'_>> = font.layout(text, scale, offset).collect();
	
	// Find the most visually pleasing width to display
	//let pixel_height = height.ceil() as usize;
	//let total_text_width = glyphs.iter().rev().map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width).next().unwrap_or(0.0).ceil() as usize;
	let total_text_width : i32 = glyphs.iter().map(|g| { if let Some(bb) = g.pixel_bounding_box() { (bb.max.x - bb.min.x) as i32 } else { 0i32 } }).sum();
	let x_offset = (image.width() as f32 - total_text_width as f32)/2f32;
	let y_offset = image.height() as f32 * 2f32 / 3f32;
	
	// By shifting our offset in the four cardinal directions, we can draw a black backdrop.
	draw_glyphs(image, &glyphs, (x_offset as i32 - 1, y_offset as i32), [0u8, 0u8, 0u8, 255u8]);
	draw_glyphs(image, &glyphs, (x_offset as i32 + 1, y_offset as i32), [0u8, 0u8, 0u8, 255u8]);
	draw_glyphs(image, &glyphs, (x_offset as i32, y_offset as i32 - 1), [0u8, 0u8, 0u8, 255u8]);
	draw_glyphs(image, &glyphs, (x_offset as i32, y_offset as i32 + 2), [0u8, 0u8, 0u8, 255u8]);
	draw_glyphs(image, &glyphs, (x_offset as i32, y_offset as i32), [255u8, 255u8, 255u8, 255u8]);
	
}

fn draw_glyphs(image:&mut ImageBuffer<Rgba<u8>, Vec<u8>>, glyphs:&Vec<PositionedGlyph>, offset:(i32, i32), font_color: [u8; 4]) {
	for g in glyphs {
		if let Some(bb) = g.pixel_bounding_box() {
			g.draw(|x, y, v| {
				// v should be in the range 0.0 to 1.0
				// so something's wrong if you get $ in the output.
				//let i = (v * mapping_scale + 0.5) as usize;
				//let c = mapping.get(i).cloned().unwrap_or(b'$');
				let x = x as i32 + bb.min.x + offset.0;
				let y = y as i32 + bb.min.y + offset.1;
				// There's still a possibility that the glyph clips the boundaries of the bitmap
				if x >= 0 && x < image.width() as i32 && y >= 0 && y < image.height() as i32 {
					let x = x as usize;
					let y = y as usize;
					let pixel = image.get_pixel_mut(x as u32, y as u32);
					let bg_color = pixel.0;
					if v > 0.0 {
						let red:u8 = ((bg_color[0] as f32 * (1.0f32 - v as f32)) + (font_color[0] as f32 * v as f32 * 255f32)).sqrt() as u8;
						let green:u8 = ((bg_color[1] as f32 * (1.0f32 - v as f32)) + (font_color[1] as f32 * v as f32 * 255f32)).sqrt() as u8;
						let blue:u8 = ((bg_color[2] as f32 * (1.0f32 - v as f32)) + (font_color[2] as f32 * v as f32 * 255f32)).sqrt() as u8;
						*pixel = image::Rgba([red, green, blue, 255u8]);
					}
				}
			})
		}
	}
}

