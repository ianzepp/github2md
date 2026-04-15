use std::collections::BTreeMap;
use std::env;
use std::process::Command;

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::cli::IssueStateArg;
use crate::error::Github2MdError;

const API_ROOT: &str = "https://api.github.com";
const PAGE_SIZE: u32 = 100;

#[derive(Debug)]
pub struct IssueExport {
    pub issues: Vec<Issue>,
    pub comments: BTreeMap<u64, Vec<IssueComment>>,
}

pub struct GitHubClient {
    http: Client,
}

impl GitHubClient {
    pub fn new() -> Result<Self, Github2MdError> {
        let token = github_token()?;
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("github2md/0.1.0"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_| Github2MdError::MissingToken)?,
        );

        let http = Client::builder().default_headers(headers).build()?;
        Ok(Self { http })
    }

    pub fn export_issues(
        &self,
        repo: &str,
        state: IssueStateArg,
        include_prs: bool,
    ) -> Result<IssueExport, Github2MdError> {
        validate_repo(repo)?;

        let mut issues: Vec<Issue> = self.paginated_get(
            &format!("/repos/{repo}/issues"),
            &[("state", state.as_api_value())],
        )?;
        if !include_prs {
            issues.retain(|issue| issue.pull_request.is_none());
        }
        issues.sort_by_key(|issue| issue.number);

        let mut comments: Vec<IssueComment> = self.paginated_get(
            &format!("/repos/{repo}/issues/comments"),
            &[],
        )?;
        comments.sort_by(|a, b| {
            a.issue_url
                .cmp(&b.issue_url)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        let mut comments_by_issue: BTreeMap<u64, Vec<IssueComment>> = BTreeMap::new();
        for comment in comments {
            let issue_number = issue_number_from_url(&comment.issue_url)?;
            comments_by_issue.entry(issue_number).or_default().push(comment);
        }

        Ok(IssueExport {
            issues,
            comments: comments_by_issue,
        })
    }

    fn paginated_get<T: DeserializeOwned>(
        &self,
        path: &str,
        extra_query: &[(&str, &str)],
    ) -> Result<Vec<T>, Github2MdError> {
        let mut page = 1u32;
        let mut out = Vec::new();

        loop {
            let url = format!("{API_ROOT}{path}");
            let mut request = self
                .http
                .get(&url)
                .query(&[("per_page", PAGE_SIZE), ("page", page)]);
            for (key, value) in extra_query {
                request = request.query(&[(*key, *value)]);
            }

            let response = request.send()?;
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().unwrap_or_default();
                return Err(Github2MdError::GitHubApi { status, url, body });
            }

            let mut batch: Vec<T> = response.json()?;
            let batch_len = batch.len();
            out.append(&mut batch);
            if batch_len < PAGE_SIZE as usize {
                break;
            }
            page += 1;
        }

        Ok(out)
    }
}

fn github_token() -> Result<String, Github2MdError> {
    if let Ok(token) = env::var("GH_TOKEN") {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    let output = Command::new("gh").args(["auth", "token"]).output();
    match output {
        Ok(result) if result.status.success() => {
            let token = String::from_utf8_lossy(&result.stdout).trim().to_string();
            if token.is_empty() {
                Err(Github2MdError::MissingToken)
            } else {
                Ok(token)
            }
        }
        _ => Err(Github2MdError::MissingToken),
    }
}

fn validate_repo(repo: &str) -> Result<(), Github2MdError> {
    let Some((owner, name)) = repo.split_once('/') else {
        return Err(Github2MdError::InvalidRepo(repo.to_string()));
    };
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return Err(Github2MdError::InvalidRepo(repo.to_string()));
    }
    Ok(())
}

pub fn issue_number_from_url(url: &str) -> Result<u64, Github2MdError> {
    url.rsplit('/')
        .next()
        .ok_or_else(|| Github2MdError::InvalidIssueUrl(url.to_string()))?
        .parse::<u64>()
        .map_err(|_| Github2MdError::InvalidIssueUrl(url.to_string()))
}

#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub state_reason: Option<String>,
    pub url: String,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub locked: bool,
    pub comments: u64,
    pub author_association: String,
    pub user: Option<User>,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub assignees: Vec<User>,
    pub milestone: Option<Milestone>,
    pub pull_request: Option<PullRequestMarker>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueComment {
    pub issue_url: String,
    pub html_url: String,
    pub body: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub author_association: String,
    pub user: Option<User>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Milestone {
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestMarker {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_number_from_issue_url() {
        let number = issue_number_from_url("https://api.github.com/repos/badlogic/pi-mono/issues/3214")
            .unwrap();
        assert_eq!(number, 3214);
    }

    #[test]
    fn rejects_invalid_repo_shape() {
        let error = validate_repo("badlogic").unwrap_err();
        assert_eq!(error.to_string(), "invalid repository 'badlogic', expected owner/name");
    }
}
