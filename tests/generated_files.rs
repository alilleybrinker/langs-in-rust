//! `README.md` is generated, and `languages.json` is written by this tool, so
//! both files in the repository have to be exactly what it produces. They
//! drifted apart once already, when languages were added to the README by hand
//! and never reached the JSON, which quietly dropped them from the list the
//! next time it was generated.

use std::fs;
use std::path::{Path, PathBuf};

use langs_in_rust::{languages, readme};

fn repo_file(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(name)
}

#[test]
fn readme_matches_the_language_list() {
    let languages = languages::load(&repo_file(languages::PATH)).unwrap();
    let committed = fs::read_to_string(repo_file(readme::PATH)).unwrap();

    assert_eq!(
        readme::render(&languages),
        committed,
        "README.md does not match languages.json; add the language there and run `cargo run -- readme`"
    );
}

#[test]
fn language_list_is_written_the_way_this_tool_writes_it() {
    let committed = fs::read_to_string(repo_file(languages::PATH)).unwrap();
    let languages = languages::load(&repo_file(languages::PATH)).unwrap();

    assert_eq!(
        languages::to_json(&languages).unwrap(),
        committed,
        "languages.json is not formatted the way this tool writes it; run `cargo run -- readme` after `cargo run -- api`"
    );
}
