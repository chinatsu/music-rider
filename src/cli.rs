use std::path::PathBuf;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(required = true, help = "Path to the directory containing FLAC files")]
    pub path: PathBuf,

    #[arg(short, long, default_value_t = 1., help = "Scale factor for the maximum level")]
    pub scale: f64,

    #[arg(short, long, default_value_t = false, action, help = "Disable bike discovery")]
    pub no_discovery: bool,

    #[arg(short, long, default_value_t = 3, action, help = "Update frequency in second")]
    pub update_frequency: i64,
}