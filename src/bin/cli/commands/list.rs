use std::{
    io::Result,
    path::PathBuf,
};

use clap::Args;

use flpak::{reader, FileType, Registry, io_error};

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct ListArgs {
    #[arg(short, long)]
    strict: bool,
    /// Archive format. Use 'list-formats' to see supported formats. If omitted, the format will be guessed.
    #[arg(short, long)]
    format: Option<String>,
    /// Path to archive
    input_file: PathBuf,
}

pub fn list(args: ListArgs) -> Result<()> {
    let registry = Registry::new();

    let rdr = registry
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
                err
            )
        })?;

    for index in 0..rdr.file_count() {
        let reader::File {
            name,
            file_type,
            size,
        } = rdr.get_file(index);

        match file_type {
            FileType::RegularFile => {
                let size = size.expect("regular file should have size");
                println!("{size:>16} {}", name.display());
            }
            FileType::Directory => println!("{}/", name.display()),
        }
    }

    Ok(())
}
