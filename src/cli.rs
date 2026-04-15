use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Parser)]
#[command(name = "github2md", about = "Export GitHub issues to Markdown")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Export issues from a GitHub repository into Markdown files.
    ExportIssues {
        /// Repository in owner/name form.
        #[arg(long)]
        repo: String,

        /// Output directory for generated Markdown files.
        #[arg(short, long)]
        output: PathBuf,

        /// Which issues to export.
        #[arg(long, default_value = "all")]
        state: IssueStateArg,

        /// Include pull requests returned by the issues endpoint.
        #[arg(long)]
        include_prs: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum IssueStateArg {
    All,
    Open,
    Closed,
}

impl IssueStateArg {
    pub fn as_api_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}
