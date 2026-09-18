//! Keeps `languages.json` and `README.md` up to date.
//!
//! ```text
//! cargo run -- api       # refresh languages.json from GitHub, then rewrite the README
//! cargo run -- readme    # rewrite the README from languages.json as it stands
//! ```

use std::env;
use std::path::Path;
use std::process::ExitCode;

use langs_in_rust::github::{self, TOKEN_VAR};
use langs_in_rust::{languages, readme, Result};

const USAGE: &str = "usage: cargo run -- [api] [readme]";

fn main() -> ExitCode {
    // Let the API token live in a `.env` file at the repository root.
    let _ = dotenvy::dotenv();

    let args: Vec<String> = env::args().skip(1).collect();

    let result = match args.as_slice() {
        [command] if command == "readme" => write_readme(),
        [command] if command == "api" => update(),
        [command] => Err(format!("unknown command: {command}\n{USAGE}").into()),
        _ => Err(USAGE.into()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Refresh `languages.json` from the GitHub API, then rewrite the README from
/// it.
fn update() -> Result<()> {
    let token = env::var(TOKEN_VAR).map_err(|_| {
        format!(
            "{TOKEN_VAR} is not set; put a GitHub API token in the environment or in a .env file"
        )
    })?;

    let languages = languages::load(Path::new(languages::PATH))?;
    let refreshed = github::refresh(&languages, &token);
    languages::store(Path::new(languages::PATH), &refreshed)?;

    write_readme()
}

/// Write the current contents of `languages.json` out to the README.
fn write_readme() -> Result<()> {
    let languages = languages::load(Path::new(languages::PATH))?;
    readme::write(Path::new(readme::PATH), &languages)
}
