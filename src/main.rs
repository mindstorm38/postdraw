//! This is a little command line utility to post process drawing scans.

use clap::{command, arg, value_parser, Command, ArgMatches};
use std::path::PathBuf;
use glam::Vec2;


fn main() -> anyhow::Result<()> {
    
    let mut matches = command!()
        .subcommand_required(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .subcommand(Command::new("bw")
            .about("Make an image black and white while erasing bright pixels and compressing black pixels range")
            .arg(arg!(<PATH> "Path of image to post process")
                .id("in_path")
                .value_parser(value_parser!(PathBuf)))
            .arg(arg!(-o --output <PATH> "Path of output image")
                .id("out_path")
                .value_parser(value_parser!(PathBuf)))
            .arg(arg!(--threshold <THRESHOLD> "Gray threshold, pixels above this are forced white, pixels below or equal are compressed to black")
                .value_parser(value_parser!(u8))
                .default_value("150"))
            .arg(arg!(--compress <FACTOR> "Compress factor to apply linearly to pixels below or equal to threshold")
                .value_parser(value_parser!(f32))
                .default_value("0.4"))
            .arg(arg!(--base <BASE> "Base gray color for all black pixels after compression")
                .value_parser(value_parser!(u8))
                .default_value("20")))
        .subcommand(Command::new("halftone")
            .about("Make and image black and white and make bright pixels transparent and create a halftone pattern from black pixels")
            .arg(arg!(<PATH> "Path of image to post process")
                .id("in_path")
                .value_parser(value_parser!(PathBuf)))
            .arg(arg!(-o --output <PATH> "Path of output image")
                .id("out_path")
                .value_parser(value_parser!(PathBuf)))
            .arg(arg!(--threshold <THRESHOLD> "Gray threshold, pixels above this are transparent, pixels below are transformed in halftone")
                .value_parser(value_parser!(u8))
                .default_value("150"))
            .arg(arg!(--stride <STRIDE> "Distance between two halftone dots")
                .value_parser(value_parser!(f32))
                .default_value("6.0"))
            .arg(arg!(--radius <RADIUS> "Radius of the circle")
                .value_parser(value_parser!(f32))
                .default_value("0.4"))
            .arg(arg!(--base <BASE> "Base gray color for all pixels, halftone only applies to alpha channel")
                .value_parser(value_parser!(u8))
                .default_value("40")))
        .get_matches();

    let (sub, sub_matches) = matches.remove_subcommand().unwrap();
    match &sub[..] {
        "bw" => bw(sub_matches),
        "halftone" => halftone(sub_matches),
        _ => unreachable!()
    }

}

fn bw(mut matches: ArgMatches) -> anyhow::Result<()> {

    let threshold = matches.remove_one::<u8>("threshold").unwrap();
    let threshold_f32 = threshold as f32;
    let compress = matches.remove_one::<f32>("compress").unwrap();
    let base = matches.remove_one::<u8>("base").unwrap();

    let in_path = matches.remove_one::<PathBuf>("in_path").unwrap();
    let out_path = matches.remove_one::<PathBuf>("out_path").unwrap_or_else(|| {
        in_path.with_extension("bw.png")
    });

    println!("Opening image...");
    println!("  Path: {in_path:?}");
    let mut image = image::open(&in_path)?.to_luma8();
    println!("  Size: {}x{}", image.width(), image.height());
    
    println!("Processing image...");
    println!("  Threshold: {threshold}");
    println!("  Compress: {compress}");
    println!("  Base: {base}");
    for pixel in image.pixels_mut() {
        if pixel[0] <= threshold {
            pixel[0] = ((pixel[0] as f32 / threshold_f32 * compress * threshold_f32) as u8).saturating_add(base);
        } else {
            pixel[0] = 255;
        }
    }

    println!("Saving image");
    println!("  Path: {out_path:?}");
    image.save(&out_path)?;

    Ok(())
    
}

fn halftone(mut matches: ArgMatches) -> anyhow::Result<()> {
    
    let threshold = matches.remove_one::<u8>("threshold").unwrap();
    let stride = matches.remove_one::<f32>("stride").unwrap();
    let radius = matches.remove_one::<f32>("radius").unwrap();
    let base = matches.remove_one::<u8>("base").unwrap();

    let in_path = matches.remove_one::<PathBuf>("in_path").unwrap();
    let out_path = matches.remove_one::<PathBuf>("out_path").unwrap_or_else(|| {
        in_path.with_extension(format!("halftone_{stride}_{radius}_{base}.png"))
    });

    println!("Opening image...");
    println!("  Path: {in_path:?}");
    let mut image = image::open(&in_path)?.to_luma_alpha8();
    println!("  Size: {}x{}", image.width(), image.height());

    println!("Processing image...");
    println!("  Threshold: {threshold}");
    println!("  Stride: {stride}");
    println!("  Radius: {radius}");
    println!("  Base: {base}");

    let angle = Vec2::from_angle(std::f32::consts::FRAC_PI_4);
    // let radius_squared = radius.powi(2);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        
        if pixel[1] != 0 && pixel[0] <= threshold {

            let radius = radius + (1.0 - pixel[0] as f32 / 255.0);
            let radius_squared = radius.powi(2);
            
            let pos = angle.rotate(Vec2::new(x as f32, y as f32));
            let index = (pos / stride).floor();
            let delta_pos = pos - index * stride;
            let delta = delta_pos / stride * 2.0 - 1.0;
            let dist_squared = delta.length_squared();
            let alpha = (radius_squared - dist_squared).clamp(0.0, 1.0);
            
            pixel[1] = (alpha * 255.0) as u8;

        } else {
            pixel[1] = 0;
        }

        pixel[0] = base;

    }

    println!("Saving image");
    println!("  Path: {out_path:?}");
    image.save(&out_path)?;

    Ok(())

}
