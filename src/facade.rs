use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use time::OffsetDateTime;

use crate::chart::{Flowchart, Node, NodeId};
use crate::github::GithubIssue;
use crate::{parse, AppResult};

const DEFAULT_PRIOR_DAYS: u16 = 7;

#[derive(Debug, Default)]
pub(crate) struct DepsArgs {
    pub title: Option<String>,
    pub all: bool,
    pub issues: Option<Vec<PathBuf>>,
    pub include_project: Option<String>,
    pub prior_days: Option<u16>,
}

pub(crate) fn build_dependencies(args: DepsArgs) -> AppResult<Flowchart> {
    let include_project_only = args.include_project;

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

    // Only show closed nodes that have been recently updated.
    let updated_after = OffsetDateTime::now_utc()
        - time::Duration::days(i64::from(
            args.prior_days.unwrap_or(DEFAULT_PRIOR_DAYS),
        ));

    let mut flowchart = Flowchart::new(
        args.title.unwrap_or_default(),
        args.all,
        include_project_only,
        Some(updated_after),
    );

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

        let project_titles = issue
            .project_items
            .iter()
            .map(|item| item.title.clone())
            .collect();

        let node = Node {
            id: id.to_string(),
            text: issue.title,
            url: issue.url,
            state: issue.state,
            labels: issue
                .labels
                .iter()
                .map(|label| label.name.clone())
                .collect(),
            project_titles,
            depends_on_urls,
            blocks_count: 0,
            updated_at: issue.updated_at,
        };
        flowchart.insert(node);

        id = id.checked_add(1).expect("Overflowed number of items");
    }

    // Update nodes to have the count of items they block.
    for (url, count) in blocks {
        let Some(blocking_node) = flowchart.get_node_by_url_mut(&url) else {
            continue;
        };
        blocking_node.blocks_count = count;
    }

    Ok(flowchart)
}
