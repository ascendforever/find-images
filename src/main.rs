use clap::Parser;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::os::unix::ffi::OsStrExt;



#[derive(Parser)]
#[structopt(about="Recursively get images (or non-images)")]
pub struct Arguments {
    #[arg(short, long,
        action=clap::ArgAction::Count, help=concat!(
        "Increase verbosity"))]
    verbose: u8,
    #[arg(short, long,
        action=clap::ArgAction::Count, help=concat!(
        "Decrease verbosity"))]
    quiet: u8,

    #[arg(short='a', long="all", help=concat!(
        "Enable processing of hidden subfiles/directories of targets"))]
    pub dohidden: bool,

    #[arg(short='n', long, help=concat!(
        "Disable sorting by last modified time"))]
    pub no_sort: bool,

    #[arg(short='0', long, help=concat!(
        "Use null as separator, not newline"))]
    pub null: bool,

    #[arg(long, help=concat!(
        "Escape paths"))]
    pub quote: bool,

    #[arg(short, long, value_name="EXT",
        value_parser, value_delimiter=',',
        default_value="dpx,exr,gif,heic,jpeg,jpg,png,svg,tiff,webp", help=concat!(
        "File extensions to filter for"))]
    pub extensions: Vec<String>,

    #[arg(short, long="output",
        default_value="-", help=concat!(
        "Output file ('-' is stdout)"))]
    pub output_file: String,

    #[arg(value_name="TARGET",
        default_value=".", help=concat!(
        "Target files and directories (recursive)"))]
    pub targets: Vec<PathBuf>,
}
impl Arguments {
    pub fn verbosity(&self) -> i16 {
        self.verbose as i16 - self.quiet as i16
    }
}



pub fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Arguments::parse();

    let verbosity = args.verbosity();

    let valid_extensions: HashSet<&str> =
        args.extensions.iter().map(|s| s.as_str()).collect();

    let mut registry = Registry::new(verbosity, valid_extensions);

    registry.populate(
        args.targets.into_iter(),
        args.dohidden
    );

    if !args.no_sort {
        registry.sort_by_modified();
    }

    match args.output_file.as_ref() {
        "-" => registry.write_all(
            &mut std::io::BufWriter::new(
                std::io::stdout().lock()
            ),
            args.null, args.quote
        )?,
        file => registry.write_all(
            &mut std::io::BufWriter::new(
                std::fs::File::create(file).map_err(
                    |e| format!("Failed to write to output file: {}", e))?
            ),
            args.null, args.quote
        )?,
    };

    Ok(())
}



pub struct Registry<'a> {
    verbosity: i16,
    registry: Vec<(std::fs::Metadata, PathBuf)>,
    valid_extensions: HashSet<&'a str>
}
impl<'a> Registry<'a> {
    pub fn new(verbosity: i16, valid_extensions: HashSet<&'a str>) -> Self {
        Self {
            verbosity,
            registry: Default::default(),
            valid_extensions
        }
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
        self.registry.sort_by_key(|(md,_)| {
            md.modified().ok().unwrap_or_else(
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

    pub fn add_file(&mut self, path: PathBuf, metadata: std::fs::Metadata) {
        if let Some(osstr_ext) = path.extension() {
            match osstr_ext.to_str() {
                Some(ext) => if self.valid_extensions.contains(ext) {
                    self.registry.push((metadata, path));
                },
                None => match self.verbosity {
                    ..0 => (),
                    0.. => eprintln!(
                        "Cannot read non-utf-8 file extension: {} on {}",
                        shlex::try_quote(&osstr_ext.to_string_lossy()).unwrap(),
                        shlex::try_quote(&path.to_string_lossy()).unwrap(),
                    ),
                },
            }
        }
    }

    pub fn add_dir(&mut self, path: PathBuf, dohidden: bool) {
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                match self.verbosity {
                    ..0 => (),
                    0.. => eprintln!("{}", e),
                }
                return;
            },
        };

        for e in entries {
            let path = match e {
                Ok(e) => e.path(),
                Err(e) => {
                    match self.verbosity {
                        ..0 => (),
                        0.. => eprintln!("{}", e),
                    }
                    continue;
                },
            };

            // unwrap is fine because it is only None if the file_name is .. or is root /
            //   (neither of which would happen in this scenario)
            if !dohidden && path.file_name().map(|name| name.to_string_lossy().starts_with('.')).unwrap_or(true) {
                continue
            }

            let metadata = match std::fs::symlink_metadata(&path) {
                Ok(md) => md,
                Err(e) => {
                    match self.verbosity {
                        ..0 => (),
                        0.. => eprintln!("{}", e),
                    }
                    continue;
                }
            };

            if metadata.file_type().is_symlink() {
                continue
            }

            if path.is_file() {
                self.add_file(path, metadata);
            } else if path.is_dir() {
                self.add_dir(path, dohidden);
            }
        }
    }
}
