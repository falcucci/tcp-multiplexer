use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use tcp_multiplexer::commands::server;
use tcp_multiplexer::dirs;

#[derive(clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(author, version, about = "Bootstrap the server", long_about = None)]
    Server,
}

#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    Json,
    Table,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        short,
        long,
        global = true,
        help = "root dir for config and data",
        env = "TCP_MULTIPLEXER_ROOT_DIR"
    )]
    root_dir: Option<PathBuf>,

    #[arg(
        short,
        long,
        global = true,
        help = "output format for command response",
        env = "TCP_MULTIPLEXER_OUTPUT_FORMAT"
    )]
    output_format: Option<OutputFormat>,
}

pub struct Context {
    pub dirs: dirs::Dirs,
    pub output_format: OutputFormat,
}

impl Context {
    pub fn for_cli(cli: &Cli) -> miette::Result<Self> {
        let dirs = dirs::Dirs::try_new(cli.root_dir.as_deref())?;
        let output_format = cli.output_format.clone().unwrap_or(OutputFormat::Table);

        Ok(Context {
            dirs,
            output_format,
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let _ = Context::for_cli(&cli)?;
    match cli.command {
        Commands::Server => server::setup().await,
    }
}
