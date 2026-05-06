use crate::parser::extract_language_tags;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Kategorien für Paketbewertung
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageType {
    MainApp,
    Plugin,
    Library,
    LanguagePack,
    Other,
}

/// Klassifiziert ein Paket anhand seines Namens in eine grobe Kategorie.
/// Diese Klassifikation wird später für das Ranking verwendet.
///
/// Kategorien:
/// - MainApp: Hauptanwendung (z.B. firefox)
/// - Addon: Varianten wie beta, nightly, developer builds
/// - LanguagePack: Sprachpakete (i18n, lang)
/// - Other: alles, was nicht erkannt wurde
pub fn classify_package(name: &str) -> PackageType {
    let name = name.to_lowercase();

    // ❗ Sprache IMMER zuletzt
    if name.contains("i18n")
        || name.contains("lang")
        || name.contains("locale")
    {
        return PackageType::LanguagePack;
    }

    // Libraries (sehr wichtig!)
    if name.contains("lib")
        || name.ends_with("-dev")
        || name.contains("sdk")
    {
        return PackageType::Library;
    }

    // Plugins/Addons
    if name.contains("plugin")
        || name.contains("addon")
        || name.contains("extension")
    {
        return PackageType::Plugin;
    }

    // Core Apps
    if name.contains("firefox")
        || name.contains("chrome")
        || name.contains("vim")
        || name.contains("nano")
    {
        return PackageType::MainApp;
    }

    PackageType::Other
}

/// Gibt eine feste Priorität für den Pakettyp zurück.
/// Höherer Wert = wichtiger / weiter oben im Ranking.
///
/// Gewichtung:
/// - MainApp: höchste Priorität (echte Hauptprogramme)
/// - Other: neutral
/// - Addon: niedriger (z.B. beta/nightly/developer builds)
/// - LanguagePack: sehr niedrig (soll meist nach unten)
pub fn type_priority(t: &PackageType) -> i64 {
    match t {
        PackageType::MainApp => 1000,
        PackageType::Other => -500,
        PackageType::Plugin => 100,
        PackageType::Library => 200,
        PackageType::LanguagePack => 500,
    }
}
/// Gibt einen Bonus-Score, wenn das Paket zur Systemsprache passt.
///
/// Beispiel:
/// - System: "de"
/// - Paket: "firefox-de" → +50
/// - Paket: "firefox-en" → kein Bonus
///
/// Wird genutzt, um sprachpassende Pakete im Ranking nach oben zu ziehen.
pub fn language_bonus(pkg: &str, lang: &str) -> i64 {
    let tags = extract_language_tags(pkg);

    // exakter Treffer = hoher Bonus
    if tags.contains(lang) {
        return 100;
    }

    // teilweise Treffer (z.B. de vs de-at Kontext)
    for t in tags {
        if t.starts_with(lang) {
            return 50;
        }
    }

    0
}

/// Berechnet Fuzzy-Ähnlichkeit zwischen Suchbegriff und Paketname
pub fn fuzzy_score(pkg: &str, name: &str) -> i64 {
    let matcher = SkimMatcherV2::default();
    matcher.fuzzy_match(name, pkg).unwrap_or(0)
}