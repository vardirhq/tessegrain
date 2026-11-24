use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};
use tessegrain::inspect::{inspect, Region};
use tessegrain::{import_image, render, Document};

#[derive(Parser)]
#[command(about = "Experimental pixel-art renderer and importer")]
struct Cli {
    #[command(subcommand)]
    command: Action,
}
#[derive(Subcommand)]
enum Action {
    Render {
        source: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value_t = 1)]
        scale: u32,
    },
    Import {
        source: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    Inspect {
        source: PathBuf,
        #[arg(long, help = "Inspect region x,y,width,height in canvas pixels")]
        region: Option<String>,
        #[arg(
            long,
            help = "Inspect only pixels matching #RRGGBB (any nonzero alpha)"
        )]
        color: Option<String>,
    },
    Validate {
        source: PathBuf,
    },
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Action::Render {
            source,
            output,
            scale,
        } => {
            if !(1..=16).contains(&scale) {
                return Err("scale must be between 1 and 16".into());
            }
            let doc: Document = serde_json::from_slice(&fs::read(source)?)?;
            let im = render(&doc)?;
            if u64::from(im.width()) * u64::from(im.height()) * u64::from(scale) * u64::from(scale)
                > 67_108_864
            {
                return Err("scaled preview is too large".into());
            }
            let im = image::imageops::resize(
                &im,
                im.width() * scale,
                im.height() * scale,
                image::imageops::FilterType::Nearest,
            );
            im.save_with_format(output, image::ImageFormat::Png)?;
        }
        Action::Import { source, output } => {
            let im = image::open(source)?.into_rgba8();
            let doc = import_image(&im)?;
            fs::write(output, format!("{}\n", serde_json::to_string_pretty(&doc)?))?;
        }
        Action::Inspect {
            source,
            region,
            color,
        } => {
            let region = region
                .map(|value| {
                    let values = value
                        .split(',')
                        .map(str::parse::<u32>)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| {
                            "region must be x,y,width,height using nonnegative integers"
                        })?;
                    if values.len() != 4 {
                        return Err("region must be x,y,width,height using nonnegative integers");
                    }
                    Ok(Region {
                        x: values[0],
                        y: values[1],
                        width: values[2],
                        height: values[3],
                    })
                })
                .transpose()?;
            let mut im = if source
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            {
                image::open(source)?.into_rgba8()
            } else {
                let doc: Document = serde_json::from_slice(&fs::read(source)?)?;
                render(&doc)?
            };
            if let Some(ref value) = color {
                let hex = value.strip_prefix('#').ok_or("color must be #RRGGBB")?;
                if !hex.is_ascii() || hex.len() != 6 {
                    return Err("color must be #RRGGBB".into());
                }
                let rgb = [
                    u8::from_str_radix(&hex[0..2], 16)?,
                    u8::from_str_radix(&hex[2..4], 16)?,
                    u8::from_str_radix(&hex[4..6], 16)?,
                ];
                for pixel in im.pixels_mut() {
                    if pixel.0[..3] != rgb {
                        pixel.0[3] = 0;
                    }
                }
            }
            let mut output = serde_json::to_value(inspect(&im, region)?)?;
            output["selection"] = serde_json::json!({"rgb": color});
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Action::Validate { source } => {
            let doc: Document = serde_json::from_slice(&fs::read(source)?)?;
            render(&doc)?;
            println!("valid");
        }
    };
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
