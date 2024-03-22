use du_dust::{draw_it, InitialDisplayData};
use std::io::Error;
use std::{cmp::max, env};
use terminal_size::{terminal_size, Width};
static DEFAULT_TERMINAL_WIDTH: usize = 80;

mod read_7z;
mod read_zip;

use clap::Parser;
use std::path::PathBuf;

/// Print compressed sizes of archives in a tree-like format.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the archive.
    archive_file: PathBuf,

    /// Maximum depth to extract nested archive.
    #[arg(short, long)]
    depth: Option<usize>,

    /// Print the actual size instead of the compressed size.
    #[arg(short = 'a', long)]
    use_actual_size: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let current_dir = env::current_dir()?;
    let resolved_zip_file = current_dir.join(args.archive_file);

    if !resolved_zip_file.exists() {
        eprintln!(
            "The specified archive does not exist: {}",
            resolved_zip_file.display()
        );
        std::process::exit(1);
    }

    let depth = args.depth.unwrap_or(usize::MAX);
    let root_node = match resolved_zip_file.extension().and_then(|ext| ext.to_str()) {
        Some("zip") => read_zip::read_zip(&resolved_zip_file, depth, args.use_actual_size)?,
        Some("7z") => read_7z::read_7z(&resolved_zip_file, depth, args.use_actual_size)?,
        Some(ext) => {
            eprintln!("Unsupported archive format: {}", ext);
            std::process::exit(1);
        }
        None => {
            eprintln!(
                "Only supports files with extensions: {}",
                resolved_zip_file.display()
            );
            std::process::exit(1);
        }
    };

    draw_it(
        InitialDisplayData {
            short_paths: false,
            is_reversed: true,
            colors_on: true,
            by_filecount: false,
            is_screen_reader: false,
            output_format: "".to_string(),
            bars_on_right: true,
        },
        false,
        get_width_of_terminal(),
        &root_node,
        false,
    );

    Ok(())
}

fn get_width_of_terminal() -> usize {
    // Simplify once https://github.com/eminence/terminal-size/pull/41 is merged
    terminal_size()
        .map(|(Width(w), _)| match cfg!(windows) {
            // Windows CI runners detect a very low terminal width
            true => max(w as usize, DEFAULT_TERMINAL_WIDTH),
            false => w as usize,
        })
        .unwrap_or(DEFAULT_TERMINAL_WIDTH)
}
