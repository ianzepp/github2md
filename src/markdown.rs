use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::error::Github2MdError;
use crate::github::{Issue, IssueComment, User};

pub fn write_issue_exports(
    output_dir: &Path,
    repository: &str,
    issues: &[Issue],
    comments: &BTreeMap<u64, Vec<IssueComment>>,
) -> Result<usize, Github2MdError> {
    let mut written = 0usize;

    for issue in issues {
        let state_dir = output_dir.join(issue.state.as_str());
        fs::create_dir_all(&state_dir)?;
        let file_name = format!("{:04}-{}.md", issue.number, slugify(&issue.title));
        let path = state_dir.join(file_name);
        let body = render_issue(
            repository,
            issue,
            comments.get(&issue.number).map(Vec::as_slice).unwrap_or(&[]),
        );
        fs::write(path, body)?;
        written += 1;
    }

    Ok(written)
}

pub fn render_issue(repository: &str, issue: &Issue, comments: &[IssueComment]) -> String {
    let mut out = String::new();

    out.push_str("---\n");
    out.push_str(&format!("repository: {}\n", yaml_scalar(repository)));
    out.push_str(&format!("number: {}\n", issue.number));
    out.push_str(&format!("title: {}\n", yaml_quoted(&issue.title)));
    out.push_str(&format!("state: {}\n", yaml_scalar(&issue.state)));
    out.push_str(&format!(
        "state_reason: {}\n",
        yaml_optional(issue.state_reason.as_deref())
    ));
    out.push_str(&format!(
        "is_pull_request: {}\n",
        issue.pull_request.is_some()
    ));
    out.push_str(&format!(
        "author: {}\n",
        yaml_scalar(user_login(issue.user.as_ref()).unwrap_or("unknown"))
    ));
    out.push_str(&format!(
        "author_association: {}\n",
        yaml_scalar(&issue.author_association)
    ));
    out.push_str(&format!("created_at: {}\n", yaml_scalar(&issue.created_at)));
    out.push_str(&format!("updated_at: {}\n", yaml_scalar(&issue.updated_at)));
    out.push_str(&format!(
        "closed_at: {}\n",
        yaml_optional(issue.closed_at.as_deref())
    ));
    out.push_str(&format!("locked: {}\n", issue.locked));
    out.push_str(&format!("comments_count: {}\n", issue.comments));
    out.push_str(&format!(
        "labels: {}\n",
        yaml_string_list(&issue.labels.iter().map(|label| label.name.as_str()).collect::<Vec<_>>())
    ));
    out.push_str(&format!(
        "assignees: {}\n",
        yaml_string_list(
            &issue
                .assignees
                .iter()
                .map(|assignee| assignee.login.as_str())
                .collect::<Vec<_>>()
        )
    ));
    out.push_str(&format!(
        "milestone: {}\n",
        yaml_optional(issue.milestone.as_ref().map(|milestone| milestone.title.as_str()))
    ));
    out.push_str(&format!("url: {}\n", yaml_scalar(&issue.html_url)));
    out.push_str(&format!("api_url: {}\n", yaml_scalar(&issue.url)));
    out.push_str("---\n\n");

    out.push_str(&format!("# Issue #{}: {}\n\n", issue.number, issue.title));
    out.push_str("## Metadata\n\n");
    out.push_str(&format!("- State: {}\n", issue.state));
    if let Some(reason) = &issue.state_reason {
        out.push_str(&format!("- State reason: {}\n", reason));
    }
    out.push_str(&format!("- Author: {}\n", user_handle(issue.user.as_ref())));
    out.push_str(&format!("- Created: {}\n", issue.created_at));
    out.push_str(&format!("- Updated: {}\n", issue.updated_at));
    if let Some(closed_at) = &issue.closed_at {
        out.push_str(&format!("- Closed: {}\n", closed_at));
    }
    out.push_str(&format!("- Locked: {}\n", issue.locked));
    out.push_str(&format!("- Comments: {}\n", issue.comments));
    out.push_str(&format!("- URL: {}\n", issue.html_url));

    if !issue.labels.is_empty() {
        let labels = issue
            .labels
            .iter()
            .map(|label| label.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("- Labels: {}\n", labels));
    }
    if !issue.assignees.is_empty() {
        let assignees = issue
            .assignees
            .iter()
            .map(|user| format!("@{}", user.login))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("- Assignees: {}\n", assignees));
    }
    if let Some(milestone) = &issue.milestone {
        out.push_str(&format!("- Milestone: {}\n", milestone.title));
    }

    out.push_str("\n## Body\n\n");
    let issue_body = issue.body.as_deref().unwrap_or("_No body_").trim();
    if issue_body.is_empty() {
        out.push_str("_No body_\n");
    } else {
        out.push_str(issue_body);
        out.push('\n');
    }

    out.push_str("\n## Comments\n\n");
    if comments.is_empty() {
        out.push_str("_No comments_\n");
        return out;
    }

    for (index, comment) in comments.iter().enumerate() {
        out.push_str(&format!(
            "### {}. {} — {}\n\n",
            index + 1,
            user_handle(comment.user.as_ref()),
            comment.created_at
        ));
        out.push_str(&format!("- Author association: {}\n", comment.author_association));
        out.push_str(&format!("- Updated: {}\n", comment.updated_at));
        out.push_str(&format!("- URL: {}\n\n", comment.html_url));
        let body = comment.body.as_deref().unwrap_or("_No body_").trim();
        if body.is_empty() {
            out.push_str("_No body_\n");
        } else {
            out.push_str(body);
            out.push('\n');
        }
        out.push('\n');
    }

    out
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
            continue;
        }

        if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        return "issue".to_string();
    }

    if slug.len() > 80 {
        slug.truncate(80);
        while slug.ends_with('-') {
            slug.pop();
        }
    }

    slug
}

fn user_handle(user: Option<&User>) -> String {
    match user {
        Some(user) => format!("@{}", user.login),
        None => "@unknown".to_string(),
    }
}

fn user_login(user: Option<&User>) -> Option<&str> {
    user.map(|user| user.login.as_str())
}

fn yaml_scalar(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }

    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':'))
    {
        return value.to_string();
    }

    yaml_quoted(value)
}

fn yaml_optional(value: Option<&str>) -> String {
    match value {
        Some(value) if !value.is_empty() => yaml_scalar(value),
        _ => "null".to_string(),
    }
}

fn yaml_quoted(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{}\"", escaped)
}

fn yaml_string_list(values: &[&str]) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }

    let items = values
        .iter()
        .map(|value| yaml_scalar(value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::{Issue, IssueComment, PullRequestMarker, User};

    fn sample_issue() -> Issue {
        Issue {
            number: 42,
            title: "Feature: export issues fast".to_string(),
            body: Some("Body text".to_string()),
            state: "open".to_string(),
            state_reason: None,
            url: "https://api.github.com/repos/example/repo/issues/42".to_string(),
            html_url: "https://github.com/example/repo/issues/42".to_string(),
            created_at: "2026-04-15T00:00:00Z".to_string(),
            updated_at: "2026-04-15T01:00:00Z".to_string(),
            closed_at: None,
            locked: false,
            comments: 1,
            author_association: "CONTRIBUTOR".to_string(),
            user: Some(User {
                login: "alice".to_string(),
            }),
            labels: Vec::new(),
            assignees: Vec::new(),
            milestone: None,
            pull_request: None::<PullRequestMarker>,
        }
    }

    #[test]
    fn slugifies_issue_titles() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("***"), "issue");
    }

    #[test]
    fn renders_comments_section() {
        let issue = sample_issue();
        let comments = vec![IssueComment {
            issue_url: "https://api.github.com/repos/example/repo/issues/42".to_string(),
            html_url: "https://github.com/example/repo/issues/42#issuecomment-1".to_string(),
            body: Some("Looks good".to_string()),
            created_at: "2026-04-15T02:00:00Z".to_string(),
            updated_at: "2026-04-15T02:00:00Z".to_string(),
            author_association: "MEMBER".to_string(),
            user: Some(User {
                login: "bob".to_string(),
            }),
        }];

        let rendered = render_issue("example/repo", &issue, &comments);
        assert!(rendered.contains("repository: example/repo"));
        assert!(rendered.contains("title: \"Feature: export issues fast\""));
        assert!(rendered.contains("author_association: CONTRIBUTOR"));
        assert!(rendered.contains("labels: []"));
        assert!(rendered.contains("assignees: []"));
        assert!(rendered.contains("# Issue #42: Feature: export issues fast"));
        assert!(rendered.contains("### 1. @bob — 2026-04-15T02:00:00Z"));
        assert!(rendered.contains("Looks good"));
    }
}
