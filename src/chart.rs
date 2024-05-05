use indexmap::{IndexMap, IndexSet};
use time::OffsetDateTime;

use crate::github::GithubIssueState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NodeId(usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
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
            && (self.is_open()
                || filter.matches_updated_after(&self.updated_at))
            && (!self.depends_on_urls.is_empty() || self.blocks_count != 0)
    }
}

#[derive(Debug)]
pub(crate) struct Filter {
    include_project_only: Option<String>,
    updated_after: Option<OffsetDateTime>,
}

impl Filter {
    fn matches_project(&self, project_titles: &IndexSet<String>) -> bool {
        self.include_project_only
            .as_ref()
            .map(|project| project_titles.contains(project))
            .unwrap_or(true)
    }

    fn matches_updated_after(&self, updated_at: &OffsetDateTime) -> bool {
        self.updated_after
            .map(|updated_after| *updated_at >= updated_after)
            .unwrap_or(false)
    }
}

#[derive(Debug, bevy::prelude::Resource)]
pub(crate) struct Flowchart {
    title: String,
    pub nodes_by_id: IndexMap<NodeId, Node>,
    pub nodes_by_url: IndexMap<String, NodeId>,
    show_all: bool,
    filter: Filter,
}

impl Flowchart {
    pub fn new(
        title: String,
        show_all: bool,
        include_project_only: Option<String>,
        updated_after: Option<OffsetDateTime>,
    ) -> Self {
        let filter = Filter {
            updated_after,
            include_project_only,
        };

        Self {
            title,
            nodes_by_id: IndexMap::default(),
            nodes_by_url: IndexMap::default(),
            show_all,
            filter,
        }
    }

    pub fn insert(&mut self, node: Node) {
        if !node.url.is_empty() {
            self.nodes_by_url.insert(node.url.clone(), node.id);
        }
        self.nodes_by_id.insert(node.id, node);
    }

    pub fn prune(&mut self) {
        // TODO: Also prune nodes_by_url.
        self.nodes_by_id.retain(|_, node| {
            self.show_all || node.passes_filter(&self.filter)
        });
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes_by_url.len()
    }

    pub fn get_node_by_url(&self, url: &str) -> Option<&Node> {
        self.nodes_by_url
            .get(url)
            .and_then(|node_id| self.nodes_by_id.get(node_id))
    }

    pub fn get_node_by_url_mut(&mut self, url: &str) -> Option<&mut Node> {
        self.nodes_by_url
            .get(url)
            .and_then(|node_id| self.nodes_by_id.get_mut(node_id))
    }

    pub fn get_node_by_id(&self, node_id: &NodeId) -> Option<&Node> {
        self.nodes_by_id.get(node_id)
    }

    pub fn get_node_by_index(&self, index: usize) -> Option<&Node> {
        self.nodes_by_id.get_index(index).map(|(_, node)| node)
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

        for node in self.nodes_by_id.values() {
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
                        self.get_node_by_url(depends_on_url.as_str())
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
