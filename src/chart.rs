use indexmap::{IndexMap, IndexSet};
use time::OffsetDateTime;

use crate::github::GithubIssueState;

pub(crate) type NodeId = String;

#[derive(Debug)]
pub(crate) struct Node {
    pub id: NodeId,
    pub text: String,
    pub url: String,
    pub state: GithubIssueState,
    #[allow(unused)]
    pub labels: Vec<String>,
    pub project_titles: IndexSet<String>,
    pub depends_on_urls: IndexSet<String>,
    pub blocks_count: u32,
    pub updated_at: OffsetDateTime,
}

impl Node {
    fn is_open(&self) -> bool {
        match self.state {
            GithubIssueState::Open => true,
            GithubIssueState::Closed => false,
        }
    }

    /// Returns true if this node should be included in the flowchart.
    fn passes_filter(&self, filter: &Filter) -> bool {
        filter.matches_project(&self.project_titles)
            && (self.is_open() || self.updated_at >= filter.after)
            && (!self.depends_on_urls.is_empty() || self.blocks_count != 0)
    }
}

#[derive(Debug)]
pub(crate) struct Filter {
    include_project_only: Option<String>,
    after: OffsetDateTime,
}

impl Filter {
    fn matches_project(&self, project_titles: &IndexSet<String>) -> bool {
        self.include_project_only
            .as_ref()
            .map(|project| project_titles.contains(project))
            .unwrap_or(true)
    }
}

#[derive(Debug)]
pub(crate) struct Flowchart {
    title: String,
    pub nodes: IndexMap<NodeId, Node>,
    show_all: bool,
    filter: Filter,
}

impl Flowchart {
    pub fn new(
        title: String,
        show_all: bool,
        include_project_only: Option<String>,
    ) -> Self {
        let filter = Filter {
            // Only show recently closed nodes.
            after: OffsetDateTime::now_utc() - time::Duration::days(30),
            include_project_only,
        };

        Self {
            title,
            nodes: IndexMap::default(),
            show_all,
            filter,
        }
    }
}

impl std::fmt::Display for Flowchart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.title.is_empty() {
            writeln!(f, "---\ntitle:{}\n---", self.title)?;
        }
        writeln!(f, "flowchart LR")?;
        // Purple border. Gray text.
        writeln!(
            f,
            "  classDef state-closed stroke:#7048D4,stroke-width:8px,color:#636871"
        )?;
        // Green border.
        writeln!(f, "  classDef state-open stroke:#317236,stroke-width:8px")?;

        for node in self.nodes.values() {
            // Does it pass the filter?
            if !self.show_all && !node.passes_filter(&self.filter) {
                continue;
            }

            write!(f, "  {}", node.id)?;
            if !node.text.is_empty() {
                write!(f, "({})", mermaid_quote(&node.text))?;
            }
            writeln!(f)?;
            match node.state {
                GithubIssueState::Open => {
                    writeln!(f, "  class {} state-open", node.id)?;
                }
                GithubIssueState::Closed => {
                    writeln!(f, "  class {} state-closed", node.id)?;
                }
            }
            if !node.url.is_empty() {
                writeln!(
                    f,
                    "  click {} {}",
                    node.id,
                    mermaid_quote(&node.url)
                )?;
            }
            if !node.depends_on_urls.is_empty() {
                for depends_on_url in &node.depends_on_urls {
                    if let Some(prerequisite) =
                        self.nodes.get(depends_on_url.as_str())
                    {
                        if self.show_all
                            || prerequisite.passes_filter(&self.filter)
                        {
                            writeln!(
                                f,
                                "  {} --> {}",
                                prerequisite.id, node.id
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// See <https://mermaid.js.org/syntax/flowchart.html#special-characters-that-break-syntax>
fn mermaid_quote(text: &str) -> String {
    format!("\"{}\"", text.replace('#', "#35;").replace('\"', "#quot;"))
}
