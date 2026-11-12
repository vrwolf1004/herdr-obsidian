//! Screen text in several languages. Sentences live in locales/<language>.json as "key": "text".
//! A {name} inside a sentence is filled from the values given to `t()`.
//!
//! Language order (the first one set wins):
//!   1. HERDR_OBSIDIAN_LANG
//!   2. "lang" in config.toml
//!   3. the OS language
//!   4. en
//!
//! A key missing from the language falls back to en, then to the key itself.

use serde_json::{Map, Value};
use std::sync::OnceLock;

use crate::config;

const DEFAULT: &str = "en";

/// Language code and file. Every supported language is listed here.
const LOCALES: [(&str, &str); 7] = [
    ("en", include_str!("../locales/en.json")),
    ("ko", include_str!("../locales/ko.json")),
    ("ru", include_str!("../locales/ru.json")),
    ("fr", include_str!("../locales/fr.json")),
    ("es", include_str!("../locales/es.json")),
    ("de", include_str!("../locales/de.json")),
    ("ja", include_str!("../locales/ja.json")),
];

type Messages = Map<String, Value>;

/// 'ko_KR.UTF-8', 'ko-KR', 'KO' become 'ko'. None when not supported.
pub fn normalize(value: &str) -> Option<&'static str> {
    let code = value.trim().replace('-', "_");
    let code = code.split(['.', '_']).next().unwrap_or("").to_lowercase();
    LOCALES
        .iter()
        .map(|(name, _)| *name)
        .find(|name| *name == code)
}

fn detect() -> &'static str {
    std::env::var("HERDR_OBSIDIAN_LANG")
        .ok()
        .and_then(|value| normalize(&value))
        .or_else(from_config)
        .or_else(|| sys_locale::get_locale().and_then(|value| normalize(&value)))
        .unwrap_or(DEFAULT)
}

fn from_config() -> Option<&'static str> {
    let path = config::config_path()?;
    let cfg = config::load(&path).ok()?;
    normalize(cfg.lang.as_deref()?)
}

fn parse(text: &str) -> Messages {
    serde_json::from_str(text).unwrap_or_default()
}

/// All locales, parsed once.
fn all() -> &'static [(&'static str, Messages)] {
    static ALL: OnceLock<Vec<(&'static str, Messages)>> = OnceLock::new();
    ALL.get_or_init(|| {
        LOCALES
            .iter()
            .map(|(name, text)| (*name, parse(text)))
            .collect()
    })
}

fn messages_for(language: &str) -> &'static Messages {
    static EMPTY: OnceLock<Messages> = OnceLock::new();
    all()
        .iter()
        .find(|(name, _)| *name == language)
        .map(|(_, messages)| messages)
        .unwrap_or_else(|| EMPTY.get_or_init(Messages::new))
}

/// The language used on screen, chosen once per run.
pub fn current() -> &'static str {
    static LANG: OnceLock<&'static str> = OnceLock::new();
    LANG.get_or_init(detect)
}

/// Chosen language first, then en, then the key itself. Empty strings count as missing.
fn resolve(chosen: &Messages, english: &Messages, key: &str) -> String {
    [chosen, english]
        .iter()
        .find_map(|messages| messages.get(key)?.as_str().filter(|text| !text.is_empty()))
        .unwrap_or(key)
        .to_string()
}

/// Fill each {name} once. An unknown {name} stays as it is, so a translation slip cannot break a line,
/// and a {name} inside a value is not filled again.
pub fn fill(text: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let value = rest.find('}').and_then(|end| {
            let (_, value) = values.iter().find(|(name, _)| *name == &rest[1..end])?;
            Some((end, *value))
        });
        match value {
            Some((end, value)) => {
                out.push_str(value);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('{');
                rest = &rest[1..];
            }
        }
    }
    out + rest
}

/// Text in the language chosen for this run.
pub fn t(key: &str, values: &[(&str, &str)]) -> String {
    fill(
        &resolve(messages_for(current()), messages_for(DEFAULT), key),
        values,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Keys that are real messages (skips "_meta.*" entries).
    fn message_keys(language: &str) -> BTreeSet<String> {
        messages_for(language)
            .keys()
            .filter(|k| !k.starts_with('_'))
            .cloned()
            .collect()
    }

    /// Sorted set of {placeholder} names in a message.
    fn placeholders(text: &str) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        let mut rest = text;
        while let Some(start) = rest.find('{') {
            let Some(end) = rest[start..].find('}') else {
                break;
            };
            names.insert(rest[start + 1..start + end].to_string());
            rest = &rest[start + end + 1..];
        }
        names
    }

    fn map(pairs: &[(&str, &str)]) -> Messages {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
            .collect()
    }

    #[test]
    fn normalizes_locale_names() {
        assert_eq!(normalize("ko_KR.UTF-8"), Some("ko"));
        assert_eq!(normalize("ko-KR"), Some("ko"));
        assert_eq!(normalize("KO"), Some("ko"));
        assert_eq!(normalize("ja_JP"), Some("ja"));
        assert_eq!(normalize("vi"), None);
    }

    #[test]
    fn fill_keeps_unknown_names_and_does_not_fill_twice() {
        assert_eq!(fill("{a} {b}", &[("a", "x")]), "x {b}");
        assert_eq!(fill("{a} {b}", &[("a", "{b}"), ("b", "y")]), "{b} y");
    }

    #[test]
    fn resolve_uses_chosen_then_english_then_key() {
        let chosen = map(&[("only.ko", "한국어"), ("both", "ko both")]);
        let english = map(&[("only.en", "english"), ("both", "en both")]);
        assert_eq!(resolve(&chosen, &english, "only.ko"), "한국어");
        assert_eq!(resolve(&chosen, &english, "only.en"), "english");
        assert_eq!(resolve(&chosen, &english, "both"), "ko both");
        assert_eq!(resolve(&chosen, &english, "missing.key"), "missing.key");
        // An empty translation counts as missing.
        let empty = map(&[("both", "")]);
        assert_eq!(resolve(&empty, &english, "both"), "en both");
    }

    #[test]
    fn every_locale_has_the_same_keys_as_english() {
        let english = message_keys("en");
        for (name, _) in LOCALES {
            assert_eq!(message_keys(name), english, "keys differ in {name}");
        }
    }

    #[test]
    fn every_locale_keeps_the_placeholders_of_english() {
        let english = messages_for("en");
        for (name, _) in LOCALES {
            let messages = messages_for(name);
            for key in message_keys("en") {
                let en_text = english[&key].as_str().unwrap();
                let text = messages[&key].as_str().unwrap();
                assert_eq!(placeholders(text), placeholders(en_text), "{name} {key}");
            }
        }
    }

    #[test]
    fn every_locale_shows_the_key_names_in_the_hint() {
        for (name, _) in LOCALES {
            let hint = messages_for(name)["hint.help"].as_str().unwrap();
            assert!(hint.contains("Ctrl+H"), "{name}");
        }
    }
}
