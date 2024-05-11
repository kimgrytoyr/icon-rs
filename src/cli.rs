use clap::{ArgAction, Parser};

#[derive(Parser)]
#[command(about, long_about = None)]
pub struct Cli {
    /// Select an icon collection by prefix. Can be used without query.
    #[clap(short = 'p', long = "prefix")]
    pub prefix: Option<String>,

    /// Generate the icon cache.
    #[clap(long = "generate-icons-cache", action=ArgAction::SetTrue)]
    pub generate_icons_cache: bool,

    /// Fetch the icon collections.
    #[clap(long = "fetch-collections", action=ArgAction::SetTrue)]
    pub fetch_collections: bool,

    /// Render previews in addition to displaying the icon identifiers.
    #[clap(long = "preview", action=ArgAction::SetTrue)]
    pub preview: bool,

    /// Open a browser to preview and select icons.
    #[clap(short = 'b', long = "browse", action=ArgAction::SetTrue)]
    pub browse: bool,

    /// Disables the browser for the current session if default browse config is set.
    #[clap(long = "no-browse", action=ArgAction::SetTrue)]
    pub no_browse: bool,

    /// Print more verbose debug messages.
    #[clap(short = 'v', long = "verbose", action=ArgAction::SetTrue)]
    pub verbose: bool,

    /// After selecting an icon in the browser, print the SVG source of the icon.
    #[clap(short = 's', long = "svg", action=ArgAction::SetTrue)]
    pub output_svg: bool,

    /// Read and print the config.
    #[clap(long = "reset-config", action=ArgAction::SetTrue)]
    pub reset_config: bool,

    /// The query string to search for.
    pub query: Option<String>,
}
