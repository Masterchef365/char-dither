use std::path::PathBuf;
use structopt::StructOpt;
use dither::prelude::*;
use anyhow::{Result, Context};
use std::str::FromStr;
use image::io::Reader as ImageReader;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Input image
    #[structopt()]
    input: PathBuf,

    /// Output debug image to the given path
    #[structopt(short = "u", long)]
    debug_path: Option<PathBuf>,

    /// Output width (height is calculated based on aspect ratio)
    #[structopt(short, long)]
    width: Option<u32>,

    /// Algorithm
    /// "floyd" | "steinberg" | "floydsteinberg" | "floyd steinberg" => FLOYD_STEINBERG,
    /// "atkinson" => ATKINSON,
    /// "stucki" => STUCKI,
    /// "burkes" => BURKES,
    /// "jarvis" | "judice" | "ninke" => JARVIS_JUDICE_NINKE,
    /// "sierra" | "sierra3" => SIERRA_3,
    #[structopt(short, long, default_value = "floyd")]
    dither_algo: String,
}

fn main() -> Result<()> {
    let cfg = Opt::from_args();

    let ditherer = Ditherer::from_str(&cfg.dither_algo)?;

    let image = ImageReader::open(&cfg.input)?.decode()?;

    let resize_width = cfg.width.unwrap_or(image.width());
    let resize_height = (image.height() * resize_width) / image.width();

    // Resize to fit the printer
    let image = image.resize(
        resize_width,
        resize_height,
        image::imageops::FilterType::Triangle,
    );

    // Convert to the ditherer's image format
    let image: Img<RGB<f64>> = Img::new(
        image.to_rgb8().pixels().map(|p| RGB::from(p.0)),
        image.width(),
    )
    .context("Image convert failed")?;

    // Dither the image
    let quantize = dither::create_quantize_n_bits_func(1)?;
    let image = image.convert_with(|rgb| rgb.to_chroma_corrected_black_and_white());
    let image = 
        ditherer
        .dither(image, quantize)
        .convert_with(RGB::from_chroma_corrected_black_and_white);

    // Convert image back to normal...
    let (width, height) = image.size();
    let image = image::RgbImage::from_raw(width, height, image.raw_buf())
        .context("Could not convert back to a regular image")?;

    if let Some(path) = cfg.debug_path {
        image.save(path)?;
    }

    Ok(())
}
