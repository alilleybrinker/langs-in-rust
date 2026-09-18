//! The language list: loading it, saving it, and putting it in table order.

use std::cmp::Reverse;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{pyjson, Result};

/// Where the language list lives, relative to the repository root.
pub const PATH: &str = "languages.json";

/// One language in the list.
///
/// The field order matches the order the fields are written in
/// `languages.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub url: String,
    /// A repository is allowed to have no description, so this can be null.
    pub description: Option<String>,
    pub stars: u64,
    /// Whether the project has been pushed to recently enough to count as
    /// active. See [`crate::github::is_active`].
    pub active: bool,
}

/// Read the language list from `path`.
pub fn load(path: &Path) -> Result<Vec<Language>> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;

    let languages = serde_json::from_str(&raw)
        .map_err(|error| format!("could not parse {}: {error}", path.display()))?;

    Ok(languages)
}

/// Write the language list back out to `path`.
pub fn store(path: &Path, languages: &[Language]) -> Result<()> {
    fs::write(path, to_json(languages)?)
        .map_err(|error| format!("could not write {}: {error}", path.display()))?;

    Ok(())
}

/// Serialize the language list the way `languages.json` is written: four-space
/// indentation, non-ASCII characters escaped.
pub fn to_json(languages: &[Language]) -> Result<String> {
    let json = pyjson::to_string(&languages)
        .map_err(|error| format!("could not serialize the language list: {error}"))?;

    Ok(json)
}

/// Split the languages into the active ones and the inactive ones, each
/// ordered by star count, most stars first.
///
/// The sort is stable, so languages with the same number of stars stay in the
/// order they appear in `languages.json`.
pub fn by_status(languages: &[Language]) -> (Vec<&Language>, Vec<&Language>) {
    let (mut active, mut inactive): (Vec<_>, Vec<_>) =
        languages.iter().partition(|language| language.active);

    active.sort_by_key(|language| Reverse(language.stars));
    inactive.sort_by_key(|language| Reverse(language.stars));

    (active, inactive)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn language(name: &str, stars: u64, active: bool) -> Language {
        Language {
            name: name.to_owned(),
            url: format!("https://github.com/example/{name}"),
            description: None,
            stars,
            active,
        }
    }

    #[test]
    fn splits_and_sorts_by_stars() {
        let languages = vec![
            language("few", 1, true),
            language("dormant", 500, false),
            language("many", 10, true),
        ];

        let (active, inactive) = by_status(&languages);

        assert_eq!(
            active.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(),
            ["many", "few"]
        );
        assert_eq!(
            inactive.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(),
            ["dormant"]
        );
    }

    #[test]
    fn keeps_file_order_for_ties() {
        let languages = vec![
            language("first", 7, true),
            language("second", 7, true),
            language("third", 7, true),
        ];

        let (active, _) = by_status(&languages);

        assert_eq!(
            active.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(),
            ["first", "second", "third"]
        );
    }
}
