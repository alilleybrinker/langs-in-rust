//! Refreshing the language list from the GitHub API.

use std::io::{self, Write};
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;
use ureq::tls::{RootCerts, TlsConfig};
use ureq::Agent;

use crate::languages::Language;
use crate::Result;

/// The environment variable holding a GitHub API token. It can also be set in
/// a `.env` file at the repository root.
pub const TOKEN_VAR: &str = "GITHUB_API_TOKEN";

/// How long a project can go without a push before the table calls it
/// inactive.
const ACTIVE_FOR: TimeDelta = TimeDelta::weeks(24);

/// How long to wait on any one API call before giving up on it.
const TIMEOUT: Duration = Duration::from_secs(30);

/// The parts of a GitHub repository the table cares about.
#[derive(Debug, Deserialize)]
struct Repository {
    description: Option<String>,
    stargazers_count: u64,
    pushed_at: DateTime<Utc>,
}

/// Fetch the latest data for every language backed by a GitHub repository.
///
/// A language whose URL is not a GitHub one, or whose request fails, is
/// carried over unchanged and reported on stderr: a network hiccup should cost
/// a language its freshness, not its place in the list.
pub fn refresh(languages: &[Language], token: &str) -> Vec<Language> {
    let agent = agent();

    let refreshed = languages
        .iter()
        .map(|language| {
            if !is_github(&language.url) {
                // TODO: Also support requests to other APIs like GitLab to get
                // this info.
                progress("Skipping...", &language.name);
                return language.clone();
            }

            progress("Updating...", &language.name);

            match fetch(&agent, language, token) {
                Ok(updated) => updated,
                Err(error) => {
                    eprintln!(
                        "\nwarning: keeping the old entry for {}: {error}",
                        language.name
                    );
                    language.clone()
                }
            }
        })
        .collect();

    // Leave the cursor on a clean line, past the progress messages.
    println!("Done! {:>30}", "");

    refreshed
}

/// Fetch one language's current data from the API.
fn fetch(agent: &Agent, language: &Language, token: &str) -> Result<Language> {
    let url = api_url(&language.url);

    let mut response = agent
        .get(&url)
        .header("Authorization", format!("token {token}"))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| format!("request to {url} failed: {error}"))?;

    let repository: Repository = response
        .body_mut()
        .read_json()
        .map_err(|error| format!("could not read the response from {url}: {error}"))?;

    Ok(Language {
        // The name and URL are ours, not the API's: the list may well call a
        // language something other than its repository name.
        name: language.name.clone(),
        url: language.url.clone(),
        description: repository.description,
        stars: repository.stargazers_count,
        active: is_active(repository.pushed_at),
    })
}

/// Whether a project pushed to at `pushed_at` still counts as active.
pub fn is_active(pushed_at: DateTime<Utc>) -> bool {
    pushed_at + ACTIVE_FOR >= Utc::now()
}

/// Whether a language's URL points at GitHub, and so can be looked up.
fn is_github(url: &str) -> bool {
    url.contains("github.com")
}

/// Turn a repository's web URL into the API URL for it.
fn api_url(url: &str) -> String {
    url.replace("github.com", "api.github.com/repos")
}

/// Build the HTTP agent used for every request.
///
/// Certificates come from the platform's own store rather than a bundle
/// compiled into the binary, so that a machine behind a proxy with its own
/// certificate authority (and `SSL_CERT_FILE` set) can still run this.
fn agent() -> Agent {
    let tls = TlsConfig::builder()
        .root_certs(RootCerts::PlatformVerifier)
        .build();

    Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .tls_config(tls)
        .build()
        .into()
}

/// Report what is happening to one language, overwriting the last report.
fn progress(action: &str, name: &str) {
    print!("{action}\t{name:<30}\r");
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_api_urls() {
        assert_eq!(
            api_url("https://github.com/rust-lang/rust"),
            "https://api.github.com/repos/rust-lang/rust"
        );
    }

    #[test]
    fn recognizes_github_urls() {
        assert!(is_github("https://github.com/rust-lang/rust"));
        assert!(!is_github("https://gitlab.com/example/language"));
    }

    #[test]
    fn a_recent_push_is_active() {
        assert!(is_active(Utc::now() - TimeDelta::weeks(1)));
        assert!(is_active(Utc::now() - ACTIVE_FOR + TimeDelta::days(1)));
    }

    #[test]
    fn an_old_push_is_inactive() {
        assert!(!is_active(Utc::now() - ACTIVE_FOR - TimeDelta::days(1)));
        assert!(!is_active(Utc::now() - TimeDelta::weeks(52)));
    }
}
