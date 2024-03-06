use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// set config file to use
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// show current config file
    #[arg(short = 'C', long)]
    pub current_config: bool,

    /// veiw previous day's notes
    #[arg(short = 'p', long, default_value_t = 0)]
    pub previous: u16,
}
