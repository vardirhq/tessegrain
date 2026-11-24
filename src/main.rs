use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};
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
