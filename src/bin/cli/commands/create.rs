use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
    path::PathBuf,
};

use clap::Args;

use flpak::{InputFileListBuilder, Registry};

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    #[arg(short, long)]
    strict: bool,
    /// Archive format. Use 'list-formats' to see supported formats.
    #[arg(short, long)]
    format: String,
    /// Input directory
    #[arg(short, long)]
    add_dir: Vec<PathBuf>,
    /// Exclude file
    #[arg(short, long)]
    exclude: Vec<String>,
    /// Output archive
    output_file: PathBuf,
}

pub fn create(args: CreateArgs) -> Result<()> {
    let registry = Registry::new();

    let mut file_list_builder = InputFileListBuilder::new();

    for dir in &args.add_dir {
        file_list_builder = file_list_builder.add_dir(dir).map_err(|err| {
            Error::new(ErrorKind::Other, format!("failed to add directory: {err}"))
        })?;
    }

    for pattern in &args.exclude {
        file_list_builder = file_list_builder.exclude_pattern(pattern);
    }

    let input_files = file_list_builder.build();

    let writer_fn = registry
        .create_writer(&args.format)
        .map_err(|err| Error::new(ErrorKind::Other, format!("failed to create archive: {err}")))?;

    writer_fn(input_files, &args.output_file, &HashMap::new())
        .map_err(|err| Error::new(ErrorKind::Other, format!("failed to create archive: {err}")))?;

    Ok(())
}
