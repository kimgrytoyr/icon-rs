use std::{collections::HashMap, error::Error};

use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode};

use crate::cli::Cli;

mod browse;
mod cli;
pub mod enums;
pub mod files;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let mut collections_cache = HashMap::<String, enums::IconCollection>::new();

    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    let config = ConfigBuilder::new().set_time_format_rfc3339().build();

    CombinedLogger::init(vec![TermLogger::new(
        log_level,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    if args.fetch_collections {
        files::fetch_collections(true)?;
    }

    if args.generate_icons_cache {
        files::generate_cached_icons()?;
    }

    if args.browse {
        browse::browse(&args, &mut collections_cache)?;
    }

    if (args.query.is_some() || args.prefix.is_some()) && !args.browse {
        let results = files::query(&args.query, &args.prefix, args.preview && !args.browse)?;

        if !args.preview {
            for r in &results {
                println!("{}", r);
            }
        }
    }

    Ok(())
}
