extern crate chrono;
extern crate shlex;
extern crate structopt;
use crate::structopt::StructOpt;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::os::unix::ffi::OsStrExt;



#[derive(StructOpt)]
#[structopt(about="Recursively get images")]
struct CLIArguments {
    #[structopt(short="a", long="all", help=concat!(
        "Enable processing of hidden subfiles/directories of targets"))]
    dohidden: bool,

    #[structopt(short="n", long, help=concat!(
        "Disable sorting by last modified time"))]
    no_sort: bool,

    #[structopt(short="0", long, help=concat!(
        "Use null as separator, not newline"))]
    null: bool,

    #[structopt(long, help=concat!(
        "Escape paths"))]
    quote: bool,

    #[structopt(short, long, value_name="EXT",
        default_value="dpx exr gif heic jpeg jpg png svg tiff webp", help=concat!(
        "File extensions to filter for"))]
    extensions: Vec<String>,

    #[structopt(value_name="TARGET", help=concat!(
        "Target files and directories (recursive)\n",
        "  If none specified, current working directory is implied"))]
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

    let valid_extensions: HashSet<&str> = args.extensions.iter().map(|s| s.as_str()).collect();

    let mut registry = Registry::new(valid_extensions);

    registry.populate(
        args.targets.into_iter().map(
            |target| PathBuf::from(&target)
        ),
        args.dohidden
    );

    if !args.no_sort {
        registry.sort_by_modified();
    }

    let mut stdout_buffer = std::io::BufWriter::new(std::io::stdout().lock());
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
        let sep = if separator_null { '\0' } else { '\n' };
        for (_, file) in &self.registry {
            match quote {
                true => writer.write_all(&shlex::bytes::try_quote(&file.as_os_str().as_bytes()).unwrap())?,
                false => writer.write_all(file.as_os_str().as_bytes())?,
            };
            write!(writer, "{}", sep)?;
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
                    shlex::try_quote(&path.to_string_lossy()).unwrap(),
                )
            }
        }
    }

    fn add_dir(&mut self, path: PathBuf, dohidden: bool) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for path in entries.filter_map(|e| e.ok() ).map(|e| e.path() ) {
                // unwrap below is safe because it is only None if the file_name is .. or is root /
                //   (neither of which would happen in this scenario)
                if !dohidden && path.file_name().map(|name| name.to_string_lossy().starts_with('.')).unwrap_or(true) {
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
