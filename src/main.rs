extern crate chrono;
extern crate shlex;
extern crate structopt;
use crate::structopt::StructOpt;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path,PathBuf};



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

    #[structopt(short, long, value_name="EXT",
                help="File extensions to filter for (default: dpx exr gif heic jpeg jpg png svg tiff webp)")]
    extensions: Vec<String>,

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

    let valid_extensions: HashSet<&str> = if args.extensions.is_empty() {
        ["dpx", "exr", "gif", "heic", "jpeg", "jpg", "png", "svg", "tiff", "webp"].into_iter().collect()
    } else {
        args.extensions.iter().map(|s| s.as_str()).collect()
    };

    let mut registry = Registry::new(valid_extensions);

    registry.populate(args.targets.into_iter().map(|target| Path::new(&target).to_path_buf() ), args.dohidden);

    if !args.no_sort {
        registry.sort_by_modified();
    }

    let stdout = std::io::stdout();
    let mut stdout_buffer = std::io::BufWriter::new(stdout.lock());
    registry.write_all(&mut stdout_buffer, args.null, args.quote)?;

    Ok(())
}



struct Registry<'a> {
    registry: Vec<(std::fs::Metadata, PathBuf)>,
    valid_extensions: HashSet<&'a str>
}
impl<'a> Registry<'a> {
    pub fn new(valid_extensions: HashSet<&'a str>) -> Self {
        Self { registry: Vec::new(), valid_extensions }
    }

    pub fn write_all(&self, writer: &mut impl Write, separator_null: bool, quote: bool) -> std::io::Result<()> {
        if separator_null {
            if quote { for (_,file) in &self.registry { write!(writer, "{}\0", shlex::try_quote(&file.to_string_lossy()).unwrap())?; } }
            else     { for (_,file) in &self.registry { write!(writer, "{}\0",                  &file.to_string_lossy()          )?; } }
        } else {
            if quote { for (_,file) in &self.registry { writeln!(writer, "{}", shlex::try_quote(&file.to_string_lossy()).unwrap())?; } }
            else     { for (_,file) in &self.registry { writeln!(writer, "{}",                  &file.to_string_lossy()          )?; } }
        }
        Ok(())
    }

    pub fn sort_by_modified(&mut self) {
        self.registry.sort_by_key(|(meta,_)| {
            meta.modified().ok().unwrap_or_else(
                || std::time::SystemTime::UNIX_EPOCH,
            )
        });
    }

    pub fn populate(&mut self, source_paths: impl Iterator<Item=PathBuf>, dohidden: bool) {
        for path in source_paths {
            if path.is_file() {
                if let Ok(metadata) = std::fs::metadata(&path) { // intentionally not symlink_metadata
                    self.add_file(path, metadata);
                }
            } else if path.is_dir() {
                self.add_dir(path, dohidden);
            }
        }
    }

    fn add_file(&mut self, path: PathBuf, metadata: std::fs::Metadata) {
        if let Some(osstr_ext) = path.extension() {
            match osstr_ext.to_str() {
                Some(ext) => if self.valid_extensions.contains(ext) {
                    self.registry.push((metadata, path));
                },
                None => eprintln!(
                    "Cannot read non-utf-8 file extension: {} on {}",
                    shlex::try_quote(&osstr_ext.to_string_lossy()).unwrap(),
                    shlex::try_quote(&path.to_string_lossy()).unwrap()
                )
            }
        }
    }

    fn add_dir(&mut self, path: PathBuf, dohidden: bool) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for path in entries.filter_map(|e| e.ok() ).map(|e| e.path() ) {
                if !dohidden && path.file_name().map(|name| name.to_string_lossy().starts_with('.')).unwrap_or(true) { // this unwraps to None if the file_name is .. or is root / (neither of which would happen in this scenario)
                    continue
                }
                if let Ok(metadata) = std::fs::symlink_metadata(&path) {
                    if metadata.file_type().is_symlink() {
                        continue
                    }
                    if path.is_file() {
                        self.add_file(path, metadata)
                    } else if path.is_dir() {
                        self.add_dir(path, dohidden);
                    }
                }
            }
        }
    }
}
