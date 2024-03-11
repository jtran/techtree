use std::process::ExitCode;

use clap::{Args, Parser};
use indexmap::{IndexMap, IndexSet};

use crate::chart::{Flowchart, Node, NodeId};
use crate::github::GithubProjectItemListResult;

mod chart;
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
    #[command(about = "Visualize dependency map")]
    Map(MapArgs),
}

#[derive(Debug, Args)]
struct MapArgs {
    #[arg(long, help = "Header markdown to output at the top of the diagram")]
    pub header: Option<String>,
    #[arg(long, help = "Mermaid diagram title")]
    pub title: Option<String>,
    #[arg(
        long,
        help = "Case-insensitive status to treat as done.  Can be used multiple times.  Default is \"Done\"."
    )]
    pub done_status: Option<Vec<String>>,
    #[arg(long, short, help = "Output all tasks; don't use default filter")]
    pub all: bool,
    #[arg(
        help = "JSON Project Items List stored in a file.  Use - for STDIN."
    )]
    pub project_file: String,
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
        Commands::Map(args) => {
            print_dependencies_map(args)?;
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn print_dependencies_map(args: MapArgs) -> AppResult<()> {
    let items_json = if args.project_file == "-" {
        // Read from STDIN.
        let stdin = std::io::stdin().lock();
        std::io::read_to_string(stdin)?
    } else {
        // Read from a file.
        std::fs::read_to_string(args.project_file)?
    };
    // Note: It's faster to read the entire file and then parse it.
    // https://github.com/serde-rs/json/issues/160
    let items: GithubProjectItemListResult =
        serde_json::from_str(items_json.as_str())?;

    let done_statuses =
        args.done_status.unwrap_or_else(|| vec!["done".to_owned()]);

    let mut flowchart =
        Flowchart::new(args.title.unwrap_or_default(), args.all);

    let mut blocks: IndexMap<NodeId, u32> = IndexMap::default();

    let mut id = 1_usize;
    for item in items.items {
        // Parse dependencies from the body text.
        let dependencies = parse::relations(
            item.content.body.as_str(),
            item.repository.as_str(),
            item.title.as_str(),
        )
        .map(|relation| relation.target.into_owned());

        // Use a set to dedupe the dependencies.
        let mut depends_on_urls = IndexSet::new();
        depends_on_urls.extend(dependencies);

        // Increment the count of all the things that block this item.
        for depends_on_url in &depends_on_urls {
            let previous_count =
                blocks.entry(depends_on_url.clone()).or_default();
            *previous_count = previous_count.saturating_add(1);
        }

        let is_done = done_statuses
            .iter()
            .any(|done_status| item.status.eq_ignore_ascii_case(done_status));

        let node = Node {
            id: id.to_string(),
            text: item.title,
            url: item.content.url,
            is_done,
            status: item.status,
            labels: item.labels,
            depends_on_urls,
            blocks_count: 0,
        };
        flowchart.nodes.insert(node.url.clone(), node);

        id = id.checked_add(1).expect("Overflowed number of items");
    }

    // Update nodes to have the count of items they block.
    for (url, count) in blocks {
        let Some(blocking_node) = flowchart.nodes.get_mut(&url) else {
            continue;
        };
        blocking_node.blocks_count = count;
    }

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
