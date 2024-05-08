use std::{error::Error, process::exit};

use clap::Parser;
use icon::generate_cached_icons;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};

use crate::cli::Cli;

mod cli;
mod enums;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let log_level = if args.verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Warn
    };

    CombinedLogger::init(vec![TermLogger::new(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    if let Some(icon_identifier) = args.preview {
        icon::preview(&icon_identifier);
        exit(0);
    }

    if args.fetch_collections {
        icon::fetch_collections(true)?;
    }

    if args.generate_icons_cache {
        generate_cached_icons()?;
    }

    if args.query.is_some() || args.prefix.is_some() {
        icon::query(&args.query, &args.prefix)?;
    }

    Ok(())
}
