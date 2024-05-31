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
