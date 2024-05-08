use std::error::Error;

use clap::Parser;
use icon::generate_cached_icons;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode};

use crate::cli::Cli;

mod cli;
mod enums;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };
    let config = ConfigBuilder::new().set_time_format_rfc3339().build();

    CombinedLogger::init(vec![TermLogger::new(
        log_level,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    if args.fetch_collections {
        icon::fetch_collections(true)?;
    }

    if args.generate_icons_cache {
        generate_cached_icons()?;
    }

    if args.query.is_some() || args.prefix.is_some() {
        icon::query(&args.query, &args.prefix, args.preview)?;
    }

    Ok(())
}
