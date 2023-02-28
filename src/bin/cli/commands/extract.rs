use std::{
    fs,
    io::{self, Result},
    path::{PathBuf, MAIN_SEPARATOR},
};

use clap::Args;

use flpak::{io_error, reader, FileType, Registry};

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
            io_error!(
                Other,
                "failed to list files for '{}': {}",
                args.input_file.display(),
                err,
            )
        })?;

    for index in 0..rdr.file_count() {
        let reader::File {
            file_type,
            name,
            size,
        } = rdr.get_file(index);

        match file_type {
            FileType::RegularFile => {
                if verbose {
                    println!("Extracting {}... ", name.display());
                }

                let file_path = args.output_dir.join(&name);
                let size = size.expect("regular file should have size");

                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(&parent).map_err(|err| {
                        io_error!(
                            Other,
                            "failed to extract file '{}': failed to create directory '{}': {}",
                            name.display(),
                            parent.display(),
                            err
                        )
                    })?;
                }

                let mut input_reader = rdr.create_file_reader(index).map_err(|err| {
                    io_error!(
                        Other,
                        "failed to extract file '{}': {}",
                        name.display(),
                        err,
                    )
                })?;

                let mut output_file = fs::File::create(&file_path).map_err(|err| {
                    io_error!(
                        Other,
                        "failed to extract file '{}': failed to create output file '{}': {}",
                        name.display(),
                        args.input_file.display(),
                        err
                    )
                })?;

                output_file.set_len(size).map_err(|err| {
                    io_error!(
                        Other,
                        "failed to extract file '{}': failed to allocate space: {}",
                        name.display(),
                        err
                    )
                })?;

                let bytes_written =
                    io::copy(&mut input_reader, &mut output_file).map_err(|err| {
                        io_error!(Other, "failed to extract file '{}': {err}", name.display(),)
                    })?;

                if bytes_written != size {
                    return Err(io_error!(
                        Other,
                        "failed to unpack file '{}': expected {size} bytes, got {bytes_written} bytes", name.display(),
                    ));
                }
            }
            FileType::Directory => {
                if verbose {
                    println!(
                        "Creating directory {}{}... ",
                        name.display(),
                        MAIN_SEPARATOR
                    );
                }

                let dir_path = args.output_dir.join(&name);
                std::fs::create_dir_all(&dir_path).map_err(|err| {
                    io_error!(
                        Other,
                        "failed to create output directory '{}': {err}",
                        name.display()
                    )
                })?;
            }
        }
    }

    Ok(())
}
