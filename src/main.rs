use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser};

use crate::facade::DepsArgs;

mod bevy_app;
mod chart;
mod facade;
mod github;
mod parse;
mod util;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(version, about = "GitHub Projects dependency analysis")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    #[command(about = "Interactive GUI")]
    Gui(GuiArgs),
    #[command(about = "Visualize dependency map")]
    Map(MapArgs),
}

#[derive(Debug, Args)]
struct GuiArgs {
    #[arg(long, short, help = "Output all tasks; don't use default filter")]
    pub all: bool,
    #[arg(
        long,
        help = "JSON Issues List stored in a file.  You can use this multiple times."
    )]
    pub issues: Option<Vec<PathBuf>>,
}

#[derive(Debug, Args)]
struct MapArgs {
    #[arg(long, help = "Header markdown to output at the top of the diagram")]
    pub header: Option<String>,
    #[arg(long, help = "Mermaid diagram title")]
    pub title: Option<String>,
    #[arg(long, short, help = "Output all tasks; don't use default filter")]
    pub all: bool,
    #[arg(
        long,
        help = "JSON Issues List stored in a file.  You can use this multiple times."
    )]
    pub issues: Option<Vec<PathBuf>>,
    #[arg(long, help = "Filter to only include given project title")]
    pub include_project: Option<String>,
    #[arg(
        long,
        help = "Additionally include closed issues that were updated in the last N days.  Default is 7 days."
    )]
    pub prior_days: Option<u16>,
}

fn main() -> ExitCode {
    let result = try_main();
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> AppResult<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Gui(args) => {
            bevy_app::main(args)?;
        }
        Commands::Map(args) => {
            print_dependencies_map(args)?;
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn print_dependencies_map(args: MapArgs) -> AppResult<()> {
    let flowchart = facade::build_dependencies(DepsArgs {
        title: args.title,
        all: args.all,
        issues: args.issues,
        include_project: args.include_project,
        prior_days: args.prior_days,
    })?;

    // Print markdown.
    if let Some(header) = args.header {
        println!("{header}");
        println!();
    }
    // spell-checker: disable-next-line
    println!("A &rarr; B means A blocks B, or B depends on A.");
    // spell-checker: disable-next-line
    println!("Press &harr; for full screen.");
    println!();
    println!("```mermaid");
    println!("{flowchart}");
    println!("```");

    Ok(())
}
