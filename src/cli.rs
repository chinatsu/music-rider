use std::path::PathBuf;

use clap::Parser;

/// audiosurf irl or something
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(required = true, help = "Path to the directory containing FLAC files")]
    pub path: PathBuf,

    #[arg(
        short,
        long,
        default_value_t = 1.,
        help = "Scale factor for the maximum level (limits the maximum level)"
    )]
    pub scale: f64,

    #[arg(
        short,
        long,
        default_value_t = false,
        action,
        help = "Disable bike discovery (enables playback without a bike, emits level changes to stdout)"
    )]
    pub no_discovery: bool,
}
