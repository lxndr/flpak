use std::{
    fs,
    io::{self, Error, ErrorKind, Result},
    path::PathBuf,
};

use clap::Args;

use flpak::{reader, FileType, Registry};

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct ExtractArgs {
    #[arg(short, long)]
    strict: bool,
    /// Archive format. Use 'list-formats' to see supported formats. If omitted, the format will be guessed.
    #[arg(short, long)]
    format: Option<String>,
    /// Path to archive
    input_file: PathBuf,
    /// Output path
    output_dir: PathBuf,
}

pub fn extract(args: ExtractArgs, verbose: bool) -> Result<()> {
    let registry = Registry::new();

    let mut rdr = registry
        .create_reader(
            args.format,
            &args.input_file,
            reader::Options {
                strict: args.strict,
            },
        )
        .map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!(
                    "failed to list files for '{}': {}",
                    args.input_file.display(),
                    err
                ),
            )
        })?;

    for index in 0..rdr.file_count() {
        let reader::File {
            file_type,
            name,
            size: _size,
        } = rdr.get_file(index);

        match file_type {
            FileType::RegularFile => {
                if verbose {
                    println!("Extracting {name}... ");
                }

                let file_path = args.output_dir.join(&name);

                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|err| {
                  Error::new(
                      ErrorKind::Other,
                      format!("failed to extract file '{}': failed to create output directory '{}': {}", name, parent.display(), err),
                  )
              })?;
                }

                let mut input_reader = rdr.create_file_reader(index).map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!(
                            "failed to extract file '{}': failed to read archived file '{}': {}",
                            name,
                            args.input_file.display(),
                            err
                        ),
                    )
                })?;

                let mut output_file = fs::File::create(file_path).map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!(
                            "failed to extract file '{}': failed to create output file '{}': {}",
                            name,
                            args.input_file.display(),
                            err
                        ),
                    )
                })?;

                io::copy(&mut input_reader, &mut output_file).map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!(
                            "failed to extract file '{name}': failed to w output file '{}': {err}",
                            args.input_file.display(),
                        ),
                    )
                })?;
            }
            FileType::Directory => {
                if verbose {
                    println!("Creating directory {name}... ");
                }

                let dir_path = args.output_dir.join(&name);
                std::fs::create_dir_all(&dir_path).map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!("failed to create output directory '{name}': {err}"),
                    )
                })?;
            }
        }
    }

    Ok(())
}
