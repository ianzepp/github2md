# github2md

Export GitHub issues into one Markdown file per issue, including issue body, comments, and useful metadata.

The exporter avoids the slow N+1 pattern of fetching each issue individually. Instead it does two paginated bulk downloads:

1. `GET /repos/:owner/:repo/issues`
2. `GET /repos/:owner/:repo/issues/comments`

It then joins comments to issues locally and writes Markdown files like:

```text
issues/
  open/
    3214-cloud-code-assist-api-returns-400-error-due-to-schema-meta-declarations.md
  closed/
    3224-feature-request-scroll-viewport-support-for-ctx-ui-custom-in-pi-tui.md
```

## Authentication

`github2md` tries, in order:

1. `GH_TOKEN`
2. `GITHUB_TOKEN`
3. `gh auth token`

That means the easiest path is usually just being logged in with the GitHub CLI.

## Install

Homebrew:

```bash
brew tap ianzepp/homebrew-tap
brew install github2md
```

Direct install:

```bash
curl -fsSL https://raw.githubusercontent.com/ianzepp/github2md/main/install.sh | bash
```

Build from source:

```bash
git clone https://github.com/ianzepp/github2md.git
cd github2md
cargo build --release
```

## Usage

Export all issues:

```bash
cargo run -- export-issues --repo badlogic/pi-mono --output ./issues
```

Export only open issues:

```bash
cargo run -- export-issues --repo badlogic/pi-mono --output ./issues --state open
```

Include pull requests too:

```bash
cargo run -- export-issues --repo badlogic/pi-mono --output ./issues --include-prs
```

## Output format

Each issue file contains:

- YAML frontmatter with repository, issue identity, state, timestamps, author, labels, assignees, milestone, and URLs
- issue number and title
- state, timestamps, labels, assignees, milestone, comment count
- canonical GitHub URL
- issue body
- all comments in chronological order
- per-comment author association and comment URL

## Release process

Releases are tag-driven through GitHub Actions.

- Push a tag like `v0.1.0`
- CI runs tests, builds release archives for macOS and Linux, creates a GitHub Release, and updates `ianzepp/homebrew-tap`
- The release workflow uses Node 24-compatible GitHub Action versions

Required GitHub repository secret:

- `TAP_GITHUB_TOKEN` — token with write access to `ianzepp/homebrew-tap`

## Build

```bash
cargo test
cargo run -- --help
```
