use clap::{ArgAction, Parser};

#[derive(Parser)]
#[command(about, long_about = None)]
pub struct Cli {
    #[clap(short = 'p', long = "prefix")]
    pub prefix: Option<String>,

    #[clap(long = "generate-icons-cache", action=ArgAction::SetTrue)]
    pub generate_icons_cache: bool,

    #[clap(long = "fetch-collections", action=ArgAction::SetTrue)]
    pub fetch_collections: bool,

    #[clap(short = 'v', long = "verbose", action=ArgAction::SetTrue)]
    pub verbose: bool,

    pub query: Option<String>,
}
