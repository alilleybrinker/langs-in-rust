//! Rendering `README.md` from the language list.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::languages::{self, Language};
use crate::Result;

/// Where the generated README lives, relative to the repository root.
pub const PATH: &str = "README.md";

/// Everything above the table: what the list is for, what gets in, and the
/// table's own header row.
const HEADER: &str = r#"# Languages Written in Rust

This is a (probably incomplete) list of languages implemented in
Rust. It is intended as a source of inspiration and comparison, and as a
directory of potentially interesting projects in this vein.

## What Can Be Included?

1. Is it a language?
2. Is it written in Rust?

Then it can be included in this list!

## List of Languages

| Name | ⭐ Stars | ☀️ Status | Description |
|:-----|:---------|:-----------|:-----------|
"#;

/// Everything below the table, up to the Markdown link definitions.
const FOOTER: &str = r#"
*: Parcel is a large project of which the JavaScript transformer (written in Rust)
is a small part. The "stars" number here reflects the whole project, which is
broader than a programming language project.

"#;

/// Render the language list and write it to `path`.
pub fn write(path: &Path, languages: &[Language]) -> Result<()> {
    fs::write(path, render(languages))
        .map_err(|error| format!("could not write {}: {error}", path.display()))?;

    Ok(())
}

/// Render the whole README: the header, the active languages, the inactive
/// ones, the footer, and the link definitions the table's `[Name]` cells point
/// at.
pub fn render(languages: &[Language]) -> String {
    let (active, inactive) = languages::by_status(languages);

    let mut out = String::from(HEADER);

    for language in active {
        out.push_str(&row(language));
    }

    for language in inactive {
        out.push_str(&row(language));
    }

    out.push_str(FOOTER);

    // The links keep the order of `languages.json` rather than the table's, so
    // that adding a language only adds a line here.
    for language in languages {
        let _ = writeln!(out, "[{}]: {}", language.name, language.url);
    }

    out
}

/// Render one row of the table.
fn row(language: &Language) -> String {
    let status = if language.active {
        "☀️ Active"
    } else {
        "🌙 Inactive"
    };

    // A missing description renders as "None", which is what the Python tool
    // this replaced wrote, and what the committed README contains for the
    // handful of repositories that have no description.
    let description = language.description.as_deref().unwrap_or("None");

    format!(
        "| [{}] | {} | {} | {} |\n",
        language.name,
        thousands(language.stars),
        status,
        description
    )
}

/// Format a star count with thousands separators: `101364` -> `101,364`.
fn thousands(stars: u64) -> String {
    let digits = stars.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, digit) in digits.char_indices() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }

        out.push(digit);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn language(name: &str, description: Option<&str>, stars: u64, active: bool) -> Language {
        Language {
            name: name.to_owned(),
            url: format!("https://github.com/example/{name}"),
            description: description.map(str::to_owned),
            stars,
            active,
        }
    }

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(7), "7");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(101_364), "101,364");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn renders_a_row_per_language() {
        assert_eq!(
            row(&language(
                "Rust",
                Some("Empowering everyone."),
                100_852,
                true
            )),
            "| [Rust] | 100,852 | ☀️ Active | Empowering everyone. |\n"
        );
        assert_eq!(
            row(&language("Move", None, 2_284, false)),
            "| [Move] | 2,284 | 🌙 Inactive | None |\n"
        );
    }

    #[test]
    fn puts_active_languages_first_and_links_last() {
        let languages = vec![
            language("Dormant", Some("Asleep."), 9_000, false),
            language("Lively", Some("Awake."), 10, true),
        ];

        let rendered = render(&languages);
        let table = rendered
            .strip_prefix(HEADER)
            .expect("the header comes first")
            .strip_suffix(
                "[Dormant]: https://github.com/example/Dormant\n\
                 [Lively]: https://github.com/example/Lively\n",
            )
            .expect("the links come last, in file order");

        assert_eq!(
            table,
            format!(
                "| [Lively] | 10 | ☀️ Active | Awake. |\n\
                 | [Dormant] | 9,000 | 🌙 Inactive | Asleep. |\n\
                 {FOOTER}"
            )
        );
    }
}
