use std::num::NonZeroU32;

use time::OffsetDateTime;

type GithubId = String;
type GithubNumber = NonZeroU32;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubProjectItemListResult {
    pub items: Vec<GithubProjectItem>,
    #[allow(unused)]
    pub total_count: u32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubProjectItem {
    pub content: GithubProjectItemContent,
    #[allow(unused)]
    pub id: GithubId,
    #[allow(unused)]
    #[serde(default)]
    pub labels: Vec<String>,
    #[allow(unused)]
    #[serde(default)]
    pub repository: String,
    #[allow(unused)]
    #[serde(default)]
    pub status: String,
    #[allow(unused)]
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubProjectItemContent {
    #[allow(unused)]
    #[serde(rename = "type")]
    pub item_type: String,
    #[allow(unused)]
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

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubIssue {
    #[serde(default)]
    pub assignees: Option<Vec<GithubIssueAssignee>>,
    #[serde(default)]
    pub body: String,
    pub closed: bool,
    #[serde(default)]
    pub comments: Option<Vec<GithubIssueComment>>,
    #[allow(unused)]
    pub id: GithubId,
    #[allow(unused)]
    pub labels: Vec<GithubLabel>,
    // TODO: Add milestone.
    // pub milestone: Option<_>,
    /// The issue or PR number that you use to reference it, e.g. #123.
    pub number: GithubNumber,
    pub project_items: Vec<GithubIssueProjectItem>,
    pub state: GithubIssueState,
    pub title: String,
    #[serde(deserialize_with = "deserialize_rfc3339")]
    pub updated_at: OffsetDateTime,
    pub url: String,
}

impl GithubIssue {
    /// Returns the repository part of the URL, e.g.
    /// "https://github.com/owner/repo", if found.
    pub fn repository(&self) -> Option<&str> {
        if let Some((repo, _)) = self.url.split_once("/issues") {
            return Some(repo);
        }

        if let Some((repo, _)) = self.url.split_once("/pull") {
            return Some(repo);
        }

        None
    }
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubIssueAssignee {
    #[allow(unused)]
    pub id: GithubId,
    pub login: String,
    pub name: String,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubIssueComment {
    #[allow(unused)]
    pub id: GithubId,
    pub author: GithubIssueCommentAuthor,
    pub author_association: String,
    pub body: String,
    pub created_at: String, // Timestamp
    pub includes_created_edit: bool,
    pub is_minimized: bool,
    pub minimized_reason: String,
    // TODO: Add reactions.
    // pub reaction_groups: Vec<_>,
    pub url: String,
    pub viewer_did_author: bool,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubIssueCommentAuthor {
    pub login: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubLabel {
    #[allow(unused)]
    pub id: GithubId,
    #[allow(unused)]
    pub name: String,
    #[serde(default)]
    #[allow(unused)]
    pub description: String,
    /// Hex color without the # prefix.
    #[allow(unused)]
    pub color: String,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubIssueProjectItem {
    /// The status field of the project item.  Since Projects are customizable,
    /// it's possible for a project to not have a status field.
    #[serde(default)]
    pub status: Option<GithubIssueProjectItemStatus>,
    /// The title of the Project that this is an item of.
    pub title: String,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubIssueProjectItemStatus {
    pub option_id: GithubId,
    /// The name of the status enum, e.g. "Not Started", "In Progress" or "Done".
    pub name: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GithubIssueState {
    #[default]
    Open,
    Closed,
}

impl<'de> serde::Deserialize<'de> for GithubIssueState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        match s.to_ascii_uppercase().as_str() {
            "OPEN" => Ok(GithubIssueState::Open),
            "CLOSED" => Ok(GithubIssueState::Closed),
            _ => Err(serde::de::Error::custom(format!(
                "Unexpected value: {s:?}"
            ))),
        }
    }
}

fn deserialize_rfc3339<'de, D>(
    deserializer: D,
) -> Result<OffsetDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    let format = time::format_description::well_known::Rfc3339;
    OffsetDateTime::parse(&s, &format).map_err(|err| {
        serde::de::Error::custom(format!(
            "Failed to parse RFC 3339 date time: {err}"
        ))
    })
}
