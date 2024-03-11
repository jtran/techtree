use std::num::NonZeroU32;

type GithubId = String;
type GithubNumber = NonZeroU32;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct GithubProjectItemListResult {
    pub items: Vec<GithubProjectItem>,
    #[serde(rename = "totalCount")]
    #[allow(unused)]
    pub total_count: u32,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct GithubProjectItem {
    pub content: GithubProjectItemContent,
    #[allow(unused)]
    pub id: GithubId,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct GithubProjectItemContent {
    #[allow(unused)]
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default)]
    pub body: String,
    #[allow(unused)]
    pub title: String,
    /// The issue or PR number that you use to reference it, e.g. #123.
    #[allow(unused)]
    #[serde(default)]
    pub number: Option<GithubNumber>,
    #[allow(unused)]
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub url: String,
}
