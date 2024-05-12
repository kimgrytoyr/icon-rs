use std::{collections::HashMap, error::Error};

use clap::Parser;
use config::read_config_file;
use log::LevelFilter;
use resvg::usvg::fontdb;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode};

use crate::{cli::Cli, files::preview};

mod browse;
mod cli;
pub mod config;
pub mod enums;
pub mod files;
mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let mut collections_cache = HashMap::<String, enums::IconCollection>::new();
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    let log_config = ConfigBuilder::new().set_time_format_rfc3339().build();

    CombinedLogger::init(vec![TermLogger::new(
        log_level,
        log_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    if args.reset_config {
        config::create_default_config_file()?;
    }

    let config = read_config_file()?;

    let browse = if let Some(default_browse) = config.default_browse {
        (args.browse || default_browse) && !args.no_browse
    } else {
        args.browse
    };

    if args.fetch_collections {
        files::fetch_collections(true)?;
    }

    if args.generate_icons_cache {
        files::generate_cached_icons()?;
    }

    if browse {
        browse::browse(&args, &mut collections_cache, &mut fontdb)?;
    }

    if (args.query.is_some() || args.prefix.is_some()) && !browse {
        let results = files::query(&args.query, &args.prefix)?;

        for r in &results {
            if args.preview {
                preview(r, &mut HashMap::new(), &mut fontdb)?;
            }
            println!("{}", r);
            if args.preview {
                println!("");
            }
        }
    }

    Ok(())
}
