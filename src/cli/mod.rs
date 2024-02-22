use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// set config file to use
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// show current config file
    #[arg(short = 'C', long)]
    pub current_config: bool,
}
