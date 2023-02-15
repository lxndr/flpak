use clap::{Parser, Subcommand};
use std::{fs, io, path::PathBuf};

use flpak::{reader, registry::Registry, FileType, InputFileListBuilder};

#[derive(Parser)]
#[command(about = "An archive utility", long_about = None)]
struct Args {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    strict: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show supported formats
    #[command()]
    ListFormats,
    /// Check archive integrity
    #[command(arg_required_else_help = true)]
    Check {
        /// Archive format
        #[arg(short, long)]
        format: Option<String>,
        /// Path to archive
        input_file: PathBuf,
    },
    /// List files
    #[command(arg_required_else_help = true)]
    List {
        /// Archive format
        #[arg(short, long)]
        format: Option<String>,
        /// Path to archive
        input_file: PathBuf,
    },
    /// Extract files
    #[command(arg_required_else_help = true)]
    Extract {
        /// Archive format
        #[arg(short, long)]
        format: Option<String>,
        /// Path to archive
        input_file: PathBuf,
        /// Output path
        output_dir: PathBuf,
    },
    /// Create archive
    #[command(arg_required_else_help = true)]
    Create {
        /// Archive format
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
    },
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let registry = Registry::new();

    match args.command {
        Commands::ListFormats => {
            for format_desc in registry.list() {
                let mut capabilities = Vec::new();

                if format_desc.make_reader_fn.is_some() {
                    capabilities.push("extract");
                }

                if format_desc.writer_fn.is_some() {
                    capabilities.push("create");
                }

                let capabilities = capabilities.join(", ");

                println!(
                    "{:<8}{:<48}{}",
                    format_desc.name, format_desc.description, capabilities
                );
            }
        }

        Commands::Check { format, input_file } => {
            let mut rdr = registry
                .create_reader(format, &input_file, reader::Options { strict: true })
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

            for index in 0..rdr.len() {
                let reader::File {
                    name, file_type, ..
                } = rdr.get_file(index);

                match file_type {
                    FileType::RegularFile => {
                        if args.verbose {
                            println!("Checking {}...", name);
                        }

                        let mut stm = rdr.open_file_by_index(index).map_err(|err| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("failed to open archived file '{}': {}", name, err),
                            )
                        })?;

                        io::copy(&mut stm, &mut io::sink()).map_err(|err| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("failed to read archived file '{}': {}", name, err),
                            )
                        })?;
                    }
                    FileType::Directory => {
                        if args.verbose {
                            println!("Checking {name}/...");
                        }
                    }
                }
            }
        }

        Commands::List { format, input_file } => {
            let rdr = registry
                .create_reader(
                    format,
                    &input_file,
                    reader::Options {
                        strict: args.strict,
                    },
                )
                .map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "failed to list files for '{}': {}",
                            input_file.display(),
                            err
                        ),
                    )
                })?;

            for index in 0..rdr.len() {
                let reader::File {
                    name,
                    file_type,
                    size,
                } = rdr.get_file(index);

                match file_type {
                    FileType::RegularFile => {
                        let size = size.unwrap();
                        println!("{name} ({size})");
                    }
                    FileType::Directory => println!("{name}/"),
                }
            }
        }

        Commands::Extract {
            format,
            input_file,
            output_dir,
        } => {
            let mut rdr = registry
                .create_reader(
                    format,
                    &input_file,
                    reader::Options {
                        strict: args.strict,
                    },
                )
                .map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "failed to list files for '{}': {}",
                            input_file.display(),
                            err
                        ),
                    )
                })?;

            for index in 0..rdr.len() {
                let reader::File {
                    name, file_type, ..
                } = rdr.get_file(index);

                match file_type {
                    FileType::RegularFile => {
                        if args.verbose {
                            println!("Extracting {name}...");
                        }

                        let file_path = output_dir.join(&name);

                        if let Some(parent) = file_path.parent() {
                            std::fs::create_dir_all(parent).map_err(|err| {
                                io::Error::new(
                                    io::ErrorKind::Other,
                                    format!("failed to extract file '{}': failed to create output directory '{}': {}", name, parent.display(), err),
                                )
                            })?;
                        }

                        let mut input_reader = rdr.open_file_by_index(index).map_err(|err| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("failed to extract file '{}': failed to read archived file '{}': {}", name, input_file.display(), err),
                            )
                        })?;

                        let mut output_file = fs::File::create(file_path).map_err(|err| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("failed to extract file '{}': failed to create output file '{}': {}", name, input_file.display(), err),
                            )
                        })?;

                        io::copy(&mut input_reader, &mut output_file).map_err(|err| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!(
                                    "failed to extract file '{}': failed to w output file '{}': {}",
                                    name,
                                    input_file.display(),
                                    err
                                ),
                            )
                        })?;
                    }
                    FileType::Directory => {
                        let dir_path = output_dir.join(&name);
                        std::fs::create_dir_all(&dir_path).map_err(|err| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("failed to create output directory '{}': {}", name, err),
                            )
                        })?;
                    }
                }
            }
        }

        Commands::Create {
            format,
            output_file,
            add_dir,
            exclude,
        } => {
            let mut file_list_builder = InputFileListBuilder::new();

            for dir in &add_dir {
                file_list_builder = file_list_builder.add_dir(dir).map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("failed to add directory: {}", err),
                    )
                })?;
            }

            for pattern in &exclude {
                file_list_builder = file_list_builder.exclude_pattern(pattern);
            }

            let input_files = file_list_builder.build();

            let writer_fn = registry.create_writer(&format).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to create archive: {}", err),
                )
            })?;

            writer_fn(input_files, &output_file).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to create archive: {}", err),
                )
            })?;
        }
    }

    Ok(())
}
