use du_dust::{InitialDisplayData, draw_it};
use std::io::Error;
use std::{cmp::max, env};
use terminal_size::{Width, terminal_size};
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

    /// The width of the terminal for output.
    #[arg(short = 'w', long)]
    terminal_width: Option<usize>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let current_dir = env::current_dir()?;
    let resolved_archive = current_dir.join(args.archive_file);

    if !resolved_archive.exists() {
        eprintln!(
            "The specified archive does not exist: {}",
            resolved_archive.display()
        );
        std::process::exit(1);
    }

    let depth = args.depth.unwrap_or(usize::MAX);
    let root_node = match resolved_archive.extension().and_then(|ext| ext.to_str()) {
        Some("zip") => read_zip::read_zip(&resolved_archive, depth, args.use_actual_size)?,
        Some("7z") => read_7z::read_7z(&resolved_archive, depth, args.use_actual_size)?,
        Some(ext) => {
            eprintln!("Unsupported archive format: {}", ext);
            std::process::exit(1);
        }
        None => {
            eprintln!(
                "Only supports files with extensions: {}",
                resolved_archive.display()
            );
            std::process::exit(1);
        }
    };

    let terminal_width = args.terminal_width.unwrap_or(get_width_of_terminal());

    draw_it(
        InitialDisplayData {
            short_paths: false,
            is_reversed: true,
            colors_on: true,
            by_filecount: false,
            is_screen_reader: false,
            output_format: "".to_string(),
            bars_on_right: false,
            by_filetime: None,
        },
        false,
        terminal_width,
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
