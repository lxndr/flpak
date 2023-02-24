use std::{
    io::{self, Error, ErrorKind, Result},
    path::PathBuf,
};

use clap::Args;

use flpak::{reader, FileType, Registry};

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct CheckArgs {
    /// Archive format. Use 'list-formats' to see supported formats. If omitted, the format will be guessed.
    #[arg(short, long)]
    format: Option<String>,
    /// Path to archive
    input_file: PathBuf,
}

pub fn check(args: CheckArgs, verbose: bool) -> Result<()> {
    let registry = Registry::new();

    let mut rdr = registry
        .create_reader(
            args.format,
            &args.input_file,
            reader::Options { strict: true },
        )
        .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;

    for index in 0..rdr.file_count() {
        let reader::File {
            name,
            file_type,
            size,
        } = rdr.get_file(index);

        match file_type {
            FileType::RegularFile => {
                if verbose {
                    println!("Checking {name}...");
                }

                let mut stm = rdr.create_file_reader(index).map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!("failed to open archived file '{name}': {err}"),
                    )
                })?;

                let bytes_written = io::copy(&mut stm, &mut io::sink()).map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!("failed to read archived file '{name}': {err}"),
                    )
                })?;

                let size = size.expect("regular file should have size");

                if bytes_written != size {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to read archived file '{name}': expected {size} bytes, got {bytes_written} bytes"),
                    ));
                }
            }
            FileType::Directory => {
                if verbose {
                    println!("Checking {name}/...");
                }
            }
        }
    }

    Ok(())
}
