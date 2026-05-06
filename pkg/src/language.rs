use std::env;

/// Hole Systemsprache für Ranking in Fuzzy-Matching
pub fn system_language() -> String {
    env::var("LANG")
        .unwrap_or_else(|_| "en".to_string())
        .chars()
        .take_while(|c| c.is_alphabetic() || *c == '-')
        .collect::<String>()
        .to_lowercase()
}