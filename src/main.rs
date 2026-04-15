use clap::Parser;

use github2md::cli::{Cli, Command};
use github2md::error::Github2MdError;
use github2md::github::{GitHubClient, IssueExport};
use github2md::markdown::write_issue_exports;

fn main() {
    let cli = Cli::parse();

    if let Err(error) = run(cli) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Github2MdError> {
    match cli.command {
        Command::ExportIssues {
            repo,
            output,
            state,
            include_prs,
        } => {
            let client = GitHubClient::new()?;
            let IssueExport { issues, comments } = client.export_issues(&repo, state, include_prs)?;
            let written = write_issue_exports(&output, &repo, &issues, &comments)?;
            eprintln!(
                "exported {} issue files for {} into {}",
                written,
                repo,
                output.display()
            );
            Ok(())
        }
    }
}
