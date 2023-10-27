extern crate chrono;
extern crate shlex;
extern crate structopt;
use std::io::Write;
use std::path::{Path,PathBuf};
use crate::structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about="Recursively get images")]
struct CLIArguments {
    #[structopt(short="a", long="all",
                help="Enable processing of hidden subfiles/directories of targets")]
    dohidden: bool,

    #[structopt(short="n", long,
                help="Disable sorting by last modified time")]
    no_sort: bool,

    #[structopt(short="0", long,
                help="Use null as separator, not newline")]
    null: bool,

    #[structopt(long,
                help="Escape paths")]
    quote: bool,

    #[structopt(value_name="TARGET",
                help="Target files and directories (recursive)\n  If none specified, current working directory is implied")]
    targets: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = {
        let mut args = CLIArguments::from_args();
        if args.targets.is_empty() {
            args.targets.push(".".to_string());
        }
        args
    };

    let mut registry: Vec<PathBuf> = Vec::new();

    args.targets.into_iter().map(|target| Path::new(&target).to_path_buf() ).for_each(
        |path| if path.is_file() {
            register_file_if_image(&mut registry, path);
        } else if path.is_dir() {
            register_dir(&mut registry, path, args.dohidden);
        }
    );

    if !args.no_sort {
        registry.sort_by_key(|entry| {
            entry.metadata().ok().and_then(|meta| meta.modified().ok()).unwrap_or_else(
                || std::time::SystemTime::UNIX_EPOCH,
            )
        });
    }

    let stdout = std::io::stdout();
    let mut stdout_buffer = std::io::BufWriter::new(stdout.lock());

    if args.null {
        if args.quote { for file in registry { write!(stdout_buffer, "{}\0", shlex::quote(&file.to_string_lossy()))?; } }
        else          { for file in registry { write!(stdout_buffer, "{}\0",              &file.to_string_lossy() )?;  } }
    } else {
        if args.quote { for file in registry { writeln!(stdout_buffer, "{}", shlex::quote(&file.to_string_lossy()))?; } }
        else          { for file in registry { writeln!(stdout_buffer, "{}",              &file.to_string_lossy() )?;  } }
    }

    Ok(())
}

fn register_file_if_image(registry: &mut Vec<PathBuf>, path: PathBuf) {
    let image_extensions: std::collections::HashSet<&str> = ["jpg", "jpeg", "png", "webp", "gif", "heic", "tiff", "dpx", "exr", "svg"].into_iter().collect();

    if let Some(osstr_ext) = path.extension() {
        match osstr_ext.to_str() {
            Some(ext) => {
                if image_extensions.contains(ext) {
                    registry.push(path);
                }
            },
            None => eprintln!(
                "Cannot read non-utf-8 file extension: {} on {}",
                shlex::quote(&osstr_ext.to_string_lossy()),
                shlex::quote(&path.to_string_lossy())
            )
        }
    }
}

fn register_dir(registry: &mut Vec<PathBuf>, path: PathBuf, dohidden: bool) {
    if let Ok(entries) = std::fs::read_dir(path) {
        for path in entries.filter_map(|e| e.ok() ).map(|e| e.path() ) {
            if !dohidden && path.file_name().map(|name| name.to_string_lossy().starts_with('.')).unwrap_or(false) {
                continue
            }
            if let Ok(metadata) = std::fs::symlink_metadata(&path) {
                if metadata.file_type().is_symlink() {
                    continue
                }
                if path.is_file() {
                    register_file_if_image(registry, path);
                } else if path.is_dir() {
                    register_dir(registry, path, dohidden);
                }
            }
        }
    }
}
