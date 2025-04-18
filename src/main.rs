#![feature(panic_payload_as_str)]

use std::{fs::OpenOptions, io::Write, path::PathBuf, process::exit, time::Duration};
use anyhow::Result;
use clap::Parser;
use console::Style;
use image::{GenericImageView, ImageReader};
use indicatif::ProgressBar;

/// Converts images to GLSL arrays
#[derive(clap::Parser, Debug)]
#[command(about, long_about)]
struct Arguments {
	/// Image file to convert
	// #[arg(default_value = "image.png")]
	input: PathBuf,

	/// Output file to write to
	// #[arg(default_value = "image.glsl")]
	output: PathBuf
}

fn run(arguments: Arguments) -> Result<()> {
	let style_heading = Style::new().underlined();
	let style_key = Style::new().bold();
	let style_value = Style::new().bold().cyan();

	let mut progress = ProgressBar::new_spinner();
	progress.set_message("Decoding image...");
	progress.enable_steady_tick(Duration::from_millis(200));

	let reader = ImageReader::open(&arguments.input)?.with_guessed_format()?;
	let format = reader.format();
	let image = reader.decode()?;
	let dimensions = image.dimensions();
	progress.finish_and_clear();

	let mut output = String::new();
	let mut output_file = OpenOptions::new()
		.create(true)
		.write(true)
		.read(false)
		.truncate(true)
		.open(&arguments.output)?
	;

	println!("{}:", style_heading.apply_to("Input file"));
	println!(
		"  - {}: {}",
		style_key.apply_to("Path"),
		style_value.apply_to(&arguments.input.to_str().unwrap())
	);
	println!(
		"  - {}: {}",
		style_key.apply_to("Format"),
		style_value.apply_to(
			match format {
				Some(value) => value.to_mime_type(),
				None => "Unknown"
			}
		)
	);
	println!(
		"  - {}: {} x {}",
		style_key.apply_to("Dimensions"),
		style_value.apply_to(dimensions.0),
		style_value.apply_to(dimensions.1)
	);
	print!("\n");
	println!("{}:", style_heading.apply_to("Output file"));
	println!(
		"  - {}: {}",
		style_key.apply_to("Path"),
		style_value.apply_to(&arguments.output.to_str().unwrap())
	);
	println!(
		"  - {}: {}",
		style_key.apply_to("Format"),
		style_value.apply_to("vec4[][], RGBA, 0..1 value range")
	);
	println!(
		"  - {}: {}",
		style_key.apply_to("Minimum OpenGL version: "),
		style_value.apply_to("Core 4.2")
	);

	let total_pixels = (dimensions.0 as u64) * (dimensions.1 as u64);
	let mut processed_pixels: u64 = 0;
	progress = ProgressBar::new(total_pixels);
	progress.set_message("Converting image...");

	output += "#version 420\n";
	output += &format!("vec4 image[{}][{}] = {{\n", dimensions.0, dimensions.1)[..];
	for x in 0..dimensions.0 {
		output += "\t{";
		for y in 0..dimensions.1 {
			let pixel = image.get_pixel(x, y).0;
			output += &format!(
				"vec4({:.7}, {:.7}, {:.7}, {:.7})",
				(1 as f64) / (255 as f64) * (pixel[0] as f64),
				(1 as f64) / (255 as f64) * (pixel[1] as f64),
				(1 as f64) / (255 as f64) * (pixel[2] as f64),
				(1 as f64) / (255 as f64) * (pixel[3] as f64)
			)[..];
			output += match (y + 1) == dimensions.1 {
				false => ", ",
				true => "}"
			};
			processed_pixels += 1;
			progress.set_position(processed_pixels);
		}
		output += match (x + 1) == dimensions.0 {
			false => ",\n",
			true => "\n};"
		}
	}

	progress.finish_and_clear();
	progress = ProgressBar::new_spinner();

	progress.set_message("Writing output file...");
	progress.enable_steady_tick(Duration::from_millis(200));

	output_file.write_all(output.as_bytes())?;

	progress.finish_and_clear();

	Ok(())
}

pub fn main() -> Result<()> {
	let style_error_heading = Style::new().underlined();
	let style_success = Style::new().bold().green();
	let style_error = Style::new().bold().red();

	if let Err(error) = run(Arguments::parse()) {
		println!("{}:\n  {}", style_error_heading.apply_to("An error occured"), style_error.apply_to(error));
		exit(1);
	} else {
		println!("\n{}", style_success.apply_to("Done!"));
		Ok(())
	}
}