//! Tooling for the list of languages written in Rust.
//!
//! `languages.json` is the source of truth: it holds every language's name and
//! URL, plus the description, star count, and activity status last seen on the
//! GitHub API. [`github::refresh`] brings that file up to date, and
//! [`readme::render`] turns it into `README.md`. Nothing else edits the
//! README, so every change to it starts as a change to the JSON.

pub mod github;
pub mod languages;
pub mod readme;

mod pyjson;

/// The result of anything that can fail here.
///
/// This is a small tool run by hand and from CI, so an error only ever needs
/// to be printed, never inspected.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
