//! Serializing JSON the way Python's `json` module does.
//!
//! `languages.json` is checked in, and for years it was written by a Python
//! tool: four-space indentation (`json.dump(..., indent=4)`) with non-ASCII
//! characters escaped as `\uXXXX` (`ensure_ascii=True`, the default). Keeping
//! that format is what makes rewriting the file show only the entries that
//! actually changed, instead of every line holding an emoji.

use std::io::{self, Write};

use serde::Serialize;
use serde_json::ser::{Formatter, PrettyFormatter, Serializer};

/// Serialize `value` as Python's `json.dump(value, file, indent=4)` would.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let mut out = Vec::new();
    let mut ser = Serializer::with_formatter(&mut out, PythonFormatter::new());
    value.serialize(&mut ser)?;
    // The serializer only ever writes UTF-8, and this formatter narrows that
    // to ASCII, so the bytes are always valid UTF-8.
    Ok(String::from_utf8(out).expect("serde_json emits UTF-8"))
}

/// A [`Formatter`] that indents like `PrettyFormatter` but escapes every
/// non-ASCII character, matching Python's `ensure_ascii=True`.
struct PythonFormatter<'a> {
    inner: PrettyFormatter<'a>,
}

impl PythonFormatter<'_> {
    fn new() -> Self {
        PythonFormatter {
            inner: PrettyFormatter::with_indent(b"    "),
        }
    }
}

impl Formatter for PythonFormatter<'_> {
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        // `serde_json` hands us the runs of characters that need no escaping
        // of their own, which is where the non-ASCII characters live. Python
        // escapes those as UTF-16, so astral characters become a surrogate
        // pair (e.g. "💎" -> "💎").
        let mut plain = fragment;

        while let Some(at) = plain.find(|c: char| !c.is_ascii()) {
            let (ascii, rest) = plain.split_at(at);
            writer.write_all(ascii.as_bytes())?;

            let c = rest.chars().next().expect("`find` located a character");
            for unit in c.encode_utf16(&mut [0; 2]) {
                write!(writer, "\\u{unit:04x}")?;
            }

            plain = &rest[c.len_utf8()..];
        }

        writer.write_all(plain.as_bytes())
    }

    // Everything else is `PrettyFormatter`'s job.

    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.begin_array(writer)
    }

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.end_array(writer)
    }

    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.begin_array_value(writer, first)
    }

    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.end_array_value(writer)
    }

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.begin_object(writer)
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.end_object(writer)
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.begin_object_key(writer, first)
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.begin_object_value(writer)
    }

    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.inner.end_object_value(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn indents_with_four_spaces() {
        let value = json!([{ "name": "Rust", "stars": 1 }]);
        let expected = "[\n    {\n        \"name\": \"Rust\",\n        \"stars\": 1\n    }\n]";
        assert_eq!(to_string(&value).unwrap(), expected);
    }

    #[test]
    fn escapes_non_ascii() {
        let value = json!("⭐️ friendly");
        assert_eq!(to_string(&value).unwrap(), r#""\u2b50\ufe0f friendly""#);
    }

    #[test]
    fn escapes_astral_characters_as_surrogate_pairs() {
        let value = json!("💎 gem");
        assert_eq!(to_string(&value).unwrap(), r#""\ud83d\udc8e gem""#);
    }

    #[test]
    fn keeps_the_usual_escapes() {
        let value = json!("a \"quote\"\n\tand a \\ backslash");
        assert_eq!(
            to_string(&value).unwrap(),
            r#""a \"quote\"\n\tand a \\ backslash""#
        );
    }

    #[test]
    fn writes_empty_containers_inline() {
        assert_eq!(to_string(&json!([])).unwrap(), "[]");
        assert_eq!(to_string(&json!({})).unwrap(), "{}");
    }
}
