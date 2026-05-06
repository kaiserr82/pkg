use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::manager::Manager;
use crate::{
    system_language,
    classify_package,
    language_bonus,
    type_priority,
};

/// Cache-Typ für Suchergebnisse
/// Key = Suchbegriff
/// Value = Liste vereinheitlichter Pakettreffer
pub type Cache = Arc<Mutex<HashMap<String, Vec<UnifiedPackage>>>>;

/// Einheitliche Darstellung eines Paket-Treffers
#[derive(Debug, Clone)]
pub struct UnifiedPackage {
    /// Paketname
    pub name: String,
    /// Paketmanager, der dieses Paket liefert
    pub manager: String,
    /// Fuzzy-Matching Score (höher = besser)
    pub score: i64,
}


/// Führt eine parallele Suche über alle Paketmanager aus
/// und nutzt einen Cache zur Performance-Optimierung
pub async fn unified_search_cached(
    managers: &[Manager],
    pkg: &str,
    cache: Cache,
) -> Vec<UnifiedPackage> {

    // --- Cache Check ---
    {
        let c = cache.lock().unwrap();
        if let Some(v) = c.get(pkg) {
            return v.clone();
        }
    }

    // --- parallele Suche starten ---
    let mut handles = Vec::new();

    for m in managers.iter().cloned() {
        let pkg = pkg.to_string();

        handles.push(tokio::spawn(async move {
            m.search(&pkg).await
        }));
    }

    // --- Ergebnisse sammeln ---
    let mut all = Vec::new();

    for h in handles {
        if let Ok(mut res) = h.await {
            all.append(&mut res);
        }
    }

    // --- Duplikate entfernen ---
    let mut map: HashMap<String, UnifiedPackage> = HashMap::new();

    for p in all {
        map.entry(p.name.clone())
            .and_modify(|e| {
                if p.score > e.score {
                    *e = p.clone();
                }
            })
            .or_insert(p);
    }

    let mut result: Vec<_> = map.into_values().collect();

    // --- Ranking ---
    let sys_lang = system_language();

    result.sort_by(|a, b| {
        let a_type = classify_package(&a.name);
        let b_type = classify_package(&b.name);

        let a_lang = language_bonus(&a.name, &sys_lang);
        let b_lang = language_bonus(&b.name, &sys_lang);

        (
            type_priority(&a_type),
            a.score + a_lang
        )
        .cmp(&(
            type_priority(&b_type),
            b.score + b_lang
        ))
    });

    // --- Cache speichern ---
    let key = pkg.to_lowercase();
    cache.lock().unwrap().insert(key, result.clone());

    result
}