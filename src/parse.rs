use std::{borrow::Cow, iter::FusedIterator};

use linkify::{LinkFinder, LinkKind};

use crate::util::regex;

/// A relation between two items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Relation<'a> {
    /// The kind of relation.
    pub kind: RelationKind,
    /// The URL of the related item.
    pub target: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationKind {
    DependsOn,
    TaskComplete,
    TaskIncomplete,
}

pub(crate) fn relations<'t, 'r, 'c>(
    text: &'t str,
    repository: &'r str,
    context: &'c str,
) -> impl FusedIterator<Item = Relation<'t>>
where
    'r: 't,
    'c: 't,
{
    // Depends on link.  Matches "Depends on:", case-insensitive, an optional
    // colon, and optional space before and after the colon.
    let depends_on_prefix = regex!(
        r"\A(?i-u)[[:space:]]*depends[[:space:]]+on[[:space:]]*:?[[:space:]]*"
    );

    text.lines().filter_map(|line| {
        // Trim since it may be arbitrarily indented.
        let line = line.trim_start();
        if line.starts_with("- [ ]") || line.starts_with("- [x]") {
            // Task list.
            let kind = if line.starts_with("- [ ]") {
                RelationKind::TaskIncomplete
            } else {
                RelationKind::TaskComplete
            };
            let task_text = line["- [ ]".len()..].trim();
            extract_url(task_text, repository)
                .map(|url| Relation { kind, target: url })
        } else if let Some(capture) = depends_on_prefix.find(line) {
            // Depends on link.  Remove "Depends on:", case-insensitive, with
            // optional space before and after the colon.
            let dep_text = &line[capture.end()..];
            resolve_url(dep_text, repository, context).map(|url| Relation {
                kind: RelationKind::DependsOn,
                target: url,
            })
        } else {
            None
        }
    })
}

/// Extract the issue URL from a string.  If a URL can't be found, a warning is
/// printed.
fn resolve_url<'a>(
    text: &'a str,
    repository: &str,
    context: &str,
) -> Option<Cow<'a, str>> {
    let url = extract_url(text, repository);

    if url.is_none() && !text.is_empty() {
        eprintln!(
            "Warning: Malformed issue or PR URL {text:?} in project item {context:?}"
        );
    }

    url
}

/// Extract the issue URL from a string.
fn extract_url<'a>(text: &'a str, repository: &str) -> Option<Cow<'a, str>> {
    if text.is_empty() {
        return None;
    }

    // Note: Regular expressions use ASCII-only matching for speed.

    // Look for a GitHub issue or PR number.
    //
    // { beginning of string or {not repo characters} }
    // # { number }
    // { end of string or {not repo characters} }
    let hash_number =
        regex!(r"(?:\A|[^0-9A-Za-z_-])#([0-9]+)(?:\z|[^0-9A-Za-z_-])");
    if let Some(captures) = hash_number.captures(text) {
        let (_, [number]) = captures.extract();
        let url = format!("{repository}/issues/{number}");
        return Some(Cow::Owned(url));
    }

    // Look for a GitHub owner/repo#number.
    let owner_repo_number =
        regex!(r"(?-u:\b)([0-9A-Za-z_-]+)/([0-9A-Za-z_-]+)#([0-9]+)(?-u:\b)");
    if let Some(captures) = owner_repo_number.captures(text) {
        let (_, [owner, repo, number]) = captures.extract();
        let url = format!("https://github.com/{owner}/{repo}/issues/{number}");
        return Some(Cow::Owned(url));
    }

    // Look for a URL.  Use only the first one.
    let is_github = regex!(r"\Ahttps?://github.com/");
    let finder = LinkFinder::new();
    for link in finder.links(text) {
        match link.kind() {
            LinkKind::Url => {
                if is_github.is_match(link.as_str()) {
                    return Some(Cow::Borrowed(link.as_str()));
                }
            }
            // Ignore email links.
            LinkKind::Email => {}
            // Ignore other, future types of links.
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_url() {
        // Note: Option<Cow<str>> compares equal if the strings are equal,
        // regardless of whether it's owned or borrowed.
        let repository = "https://github.com/foo/bar";

        // Issue number.
        assert_eq!(
            extract_url("#123", repository),
            Some(Cow::from("https://github.com/foo/bar/issues/123"))
        );
        // Issue number with leading space.
        assert_eq!(
            extract_url(" #123", repository),
            Some(Cow::from("https://github.com/foo/bar/issues/123"))
        );
        // Issue number with trailing space.
        assert_eq!(
            extract_url("#123 ", repository),
            Some(Cow::from("https://github.com/foo/bar/issues/123"))
        );
        // Trailing period.
        assert_eq!(
            extract_url("#123.", repository),
            Some(Cow::from("https://github.com/foo/bar/issues/123"))
        );
        // Owner/repo#number.
        assert_eq!(
            extract_url("aaa/bbb#123", repository),
            Some(Cow::from("https://github.com/aaa/bbb/issues/123"))
        );
        // Owner/repo#number with leading space.
        assert_eq!(
            extract_url(" aaa/bbb#123", repository),
            Some(Cow::from("https://github.com/aaa/bbb/issues/123"))
        );
        // Owner/repo#number with trailing space.
        assert_eq!(
            extract_url("aaa/bbb#123 ", repository),
            Some(Cow::from("https://github.com/aaa/bbb/issues/123"))
        );
        // Owner/repo#number with trailing period.
        assert_eq!(
            extract_url("aaa/bbb#123.", repository),
            Some(Cow::from("https://github.com/aaa/bbb/issues/123"))
        );
        // Full URL.
        let actual =
            extract_url("https://github.com/aaa/bbb/issues/123", repository);
        assert_eq!(
            actual,
            Some(Cow::from("https://github.com/aaa/bbb/issues/123"))
        );
        assert!(matches!(actual, Some(Cow::Borrowed(_))));
        // Full URL with trailing period.
        let actual =
            extract_url("https://github.com/aaa/bbb/issues/123.", repository);
        assert_eq!(
            actual,
            Some(Cow::from("https://github.com/aaa/bbb/issues/123"))
        );
        assert!(matches!(actual, Some(Cow::Borrowed(_))));
        // Full URL in markdown link.
        let actual = extract_url(
            "[text](https://github.com/aaa/bbb/issues/123)",
            repository,
        );
        assert_eq!(
            actual,
            Some(Cow::from("https://github.com/aaa/bbb/issues/123"))
        );
        assert!(matches!(actual, Some(Cow::Borrowed(_))));
    }
}
