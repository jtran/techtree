use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser};
use indexmap::{IndexMap, IndexSet};

use crate::chart::{Flowchart, Node, NodeId};
use crate::github::{GithubIssue, GithubProjectItemListResult};

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
    #[arg(long, short, help = "Output all tasks; don't use default filter")]
    pub all: bool,
    #[arg(
        long,
        help = "JSON Issues List stored in a file.  You can use this multiple times."
    )]
    pub issues: Option<Vec<PathBuf>>,
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

    let issues: Vec<GithubIssue> = args
        .issues
        .unwrap_or_default()
        .iter()
        .map(|path| {
            let issues_json_result = if path == Path::new("-") {
                // Read from STDIN.
                let stdin = std::io::stdin().lock();
                std::io::read_to_string(stdin)
            } else {
                // Read from a file.
                std::fs::read_to_string(path)
            };
            let issues_json = match issues_json_result {
                Ok(i) => i,
                Err(error) => {
                    let boxed: Box<dyn std::error::Error> = Box::new(error);
                    return Err(boxed);
                }
            };
            // Note: It's faster to read the entire file and then parse it.
            // https://github.com/serde-rs/json/issues/160
            serde_json::from_str::<Vec<GithubIssue>>(&issues_json)
                .map_err(|err| Box::new(err).into())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    let mut flowchart =
        Flowchart::new(args.title.unwrap_or_default(), args.all);

    let mut blocks: IndexMap<NodeId, u32> = IndexMap::default();

    let mut id = 1_usize;

    for issue in issues {
        // Use a set to dedupe the dependencies.
        let mut depends_on_urls = IndexSet::new();

        if let Some(repository) = issue.repository() {
            // Parse dependencies from the body text.
            let dependencies = parse::relations(
                issue.body.as_str(),
                repository,
                issue.title.as_str(),
            )
            .map(|relation| relation.target.into_owned());

            depends_on_urls.extend(dependencies);

            // Increment the count of all the things that block this item.
            for depends_on_url in &depends_on_urls {
                let previous_count =
                    blocks.entry(depends_on_url.clone()).or_default();
                *previous_count = previous_count.saturating_add(1);
            }
        } else {
            eprintln!("Warning: Unexpected issue URL; couldn't parse repository: {:?}", issue.url);
        }

        let node = Node {
            id: id.to_string(),
            text: issue.title,
            url: issue.url,
            state: issue.state,
            is_in_project: false,
            labels: issue
                .labels
                .iter()
                .map(|label| label.name.clone())
                .collect(),
            depends_on_urls,
            blocks_count: 0,
            updated_at: issue.updated_at,
        };
        flowchart.nodes.insert(node.url.clone(), node);

        id = id.checked_add(1).expect("Overflowed number of items");
    }

    for item in items.items {
        let url = item.content.url;
        let Some(node) = flowchart.nodes.get_mut(&url) else {
            // This item is not in the issues.
            continue;
        };

        node.is_in_project = true;
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
