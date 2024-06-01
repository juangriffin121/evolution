use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "blobworld")]
#[command(about = "A CLI for managing a world of blobs", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Save {
        filename: String,
    },
    Load {
        filename: String,
    },
    LoadAndSave {
        input_filename: String,
        output_filename: String,
    },
}

pub fn parse_command(cli_command: Option<Commands>) -> (Option<String>, Option<String>) {
    if let Some(command) = cli_command {
        match command {
            Commands::Save { filename } => (None, Some(filename)),
            Commands::Load { filename } => (Some(filename), None),
            Commands::LoadAndSave {
                input_filename,
                output_filename,
            } => (Some(input_filename), Some(output_filename)),
        }
    } else {
        (None, None)
    }
}
