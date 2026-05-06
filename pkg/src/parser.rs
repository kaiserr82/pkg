use std::collections::HashSet;

/// Extrahiert mögliche Sprach-Tags aus einem Paketnamen.
/// Unterstützt typische Paketmanager-Schemata wie:
/// - firefox-i18n-de
/// - firefox-lang-de
/// - firefox-de-at
pub fn extract_language_tags(name: &str) -> HashSet<String> {
    let name = name.to_lowercase();
    let mut tags = HashSet::new();

    for p in name.split(&['-', '_', '.'][..]) {
        if p.len() == 2 && p.chars().all(|c| c.is_ascii_lowercase()) {
            tags.insert(p.to_string());
        }
    }

    tags
}