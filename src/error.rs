use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Github2MdError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("GitHub API returned {status} for {url}: {body}")]
    GitHubApi {
        status: reqwest::StatusCode,
        url: String,
        body: String,
    },

    #[error("invalid repository '{0}', expected owner/name")]
    InvalidRepo(String),

    #[error("could not discover a GitHub token from GH_TOKEN, GITHUB_TOKEN, or `gh auth token`")]
    MissingToken,

    #[error("failed to parse issue number from URL: {0}")]
    InvalidIssueUrl(String),

    #[error("unexpected path without file name: {0}")]
    InvalidOutputPath(PathBuf),
}
