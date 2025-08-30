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
        default_value_t = String::from("lufs"),
        help = "sound analyzer type"
    )]
    pub analyzer: String,

    #[arg(
        long,
        default_value_t = false,
        action,
        help = "Disable exercise equipment discovery (enables playback without an exercise equipment, emits level changes to stdout)"
    )]
    pub no_discovery: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        action,
        help = "Enable debug output"
    )]
    pub debug: bool,

    #[arg(
        short,
        long,
        default_value_t = 50,
        help = "Maximum level to set on the exercise equipment"
    )]
    pub max_level: i16,

    #[arg(
        long,
        default_value_t = false,
        action,
        help = "Disable reading data from exercise equipment (still writes to it)"
    )]
    pub no_read: bool,

    #[arg(
        short,
        long,
        default_value_t = String::from("28"),
        help = "exercise equipment type"
    )]
    pub exercise_equipment_type: String,

    #[arg(short, long, default_value_t = 20., help = "song offset (in ms)")]
    pub offset: f32,
}
