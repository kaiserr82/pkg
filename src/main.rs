use dialoguer::{theme::ColorfulTheme, Select, Confirm};
use std::env;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod parser;
mod ranking;
mod manager;
mod language;
mod search;
mod sort;

use language::system_language;
use ranking::{classify_package, type_priority, language_bonus};
use manager::{get_managers};
use search::{unified_search_cached, Cache};
use sort::sorter_priority;


/// Fragt den Benutzer nach Bestätigung (Ja/Nein)
fn confirm_action(prompt: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(false)
        .interact()
        .unwrap_or(false)
}

#[tokio::main]
async fn main() {
    let cache: Cache = Arc::new(Mutex::new(HashMap::new()));

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: pkg <install|remove|search|update|list|version> [package...]");
        return;
    }

    let action = &args[1];

    // Alle Paketnamen ab Argument 2 übernehmen
    let packages: Vec<String> = args.iter().skip(2).cloned().collect();

    let managers = get_managers();

    if managers.is_empty() {
        println!("Keine Package Manager gefunden.");
        return;
    }

    match action.as_str() {

        // =========================
        // VERSION
        // =========================
        "version" | "v" | "-v" | "-version" => {
            let version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
            println!("Version: {}", version);
        }

        // =========================
        // SEARCH
        // =========================
        "search" | "s" | "-s" | "-search" => {

            if packages.is_empty() {
                println!("Bitte Paketnamen angeben.");
                return;
            }

            for pkg in &packages {

                println!("\n=== Suche nach '{}' ===", pkg);

                let results_before =
                    unified_search_cached(&managers, pkg, cache.clone()).await;

                let mut results = sorter_priority(results_before);

                // 🔥 exakte lowercase Treffer nach ganz oben
                let query = pkg.to_lowercase();

                results.sort_by(|a, b| {
                    let a_exact = a.name.to_lowercase() == query;
                    let b_exact = b.name.to_lowercase() == query;

                    b_exact.cmp(&a_exact)
                });

                if results.is_empty() {
                    println!("Keine Ergebnisse gefunden.");
                    continue;
                }

                // 🔥 nur exakter Treffer → direkt installieren
                if results.len() == 1
                    || results[0].name.to_lowercase() == query
                {
                    let best = &results[0];

                    if let Some(manager) =
                        managers.iter().find(|m| m.name == best.manager)
                    {
                        if confirm_action(&format!(
                            "'{}' über '{}' installieren?",
                            best.name,
                            best.manager
                        )) {
                            manager.install(&best.name).await;
                        }
                    }

                    continue;
                }

                let items: Vec<String> = results
                    .iter()
                    .map(|r| {
                        format!(
                            "{} ({}) score:{}",
                            r.name,
                            r.manager,
                            r.score
                        )
                    })
                    .collect();

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "Paket auswählen für '{}'",
                        pkg
                    ))
                    .items(&items)
                    .default(0)
                    .interact_opt()
                    .unwrap();

                if let Some(i) = selection {
                    let selected = &results[i];

                    if let Some(manager) =
                        managers.iter().find(|m| m.name == selected.manager)
                    {
                        if confirm_action(&format!(
                            "'{}' über '{}' installieren?",
                            selected.name,
                            selected.manager
                        )) {
                            manager.install(&selected.name).await;
                        }
                    }
                }
            }
        }

        // =========================
        // INSTALL
        // =========================
        "install" | "i" | "-i" | "-install" => {

            if packages.is_empty() {
                println!("Bitte Paketnamen angeben.");
                return;
            }

            for pkg in &packages {

                let results_before =
                    unified_search_cached(&managers, pkg, cache.clone()).await;

                let results = sorter_priority(results_before);

                if results.is_empty() {
                    println!("'{}' nicht gefunden.", pkg);
                    continue;
                }

                let best = &results[0];

                let manager = managers
                    .iter()
                    .find(|m| m.name == best.manager);

                if let Some(manager) = manager {

                    if confirm_action(&format!(
                        "'{}' mit '{}' installieren?",
                        best.real_name,
                        manager.name
                    )) {
                        manager.install(&best.real_name).await;
                    }
                }
            }
        }

        // =========================
        // REMOVE
        // =========================
        "remove" | "r" | "-r" | "-remove" => {
            if packages.is_empty() {
                println!("Bitte Paketnamen angeben.");
                return;
            }

            for pkg in &packages {
                // Suche durchführen, um den echten Paketnamen zu erhalten
                let results_before =
                    unified_search_cached(&managers, pkg, cache.clone()).await;

                let results = sorter_priority(results_before);

                if results.is_empty() {
                    println!("'{}' nicht gefunden.", pkg);
                    continue;
                }

                // Nur installierte Treffer behalten
                let mut installed = Vec::new();

                for result in results {
                    if let Some(manager) =
                        managers.iter().find(|m| m.name == result.manager)
                    {
                        if manager.is_installed(&result.real_name).await {
                            installed.push(result);
                        }
                    }
                }

                if installed.is_empty() {
                    println!("'{}' ist nicht installiert.", pkg);
                    continue;
                }

                // Besten installierten Treffer verwenden
                let best = &installed[0];

                let manager = managers
                    .iter()
                    .find(|m| m.name == best.manager)
                    .expect("Manager nicht gefunden");

                if confirm_action(&format!(
                    "'{}' über '{}' entfernen?",
                    best.real_name,
                    manager.name
                )) {
                    manager.remove(&best.real_name).await;
                }
            }
        }

        // =========================
        // UPDATE
        // =========================
        "update" | "u" | "-u" | "-update" => {
            // Kein Paket angegeben -> komplettes System aktualisieren
            if packages.is_empty() {
                if !confirm_action("Alle Paketmanager und Pakete aktualisieren?") {
                    println!("Abgebrochen.");
                    return;
                }

                for m in &managers {
                    println!("=== {} ===", m.name);
                    m.update().await;
                    m.upgrade().await;
                }

                println!("Update abgeschlossen.");
                return;
            }
            else {
                // Einzelne Pakete aktualisieren
                for pkg in &packages {
                    let results_before =
                        unified_search_cached(&managers, pkg, cache.clone()).await;

                    let results = sorter_priority(results_before);

                    if results.is_empty() {
                        println!("'{}' nicht gefunden.", pkg);
                        continue;
                    }

                    // Nur installierte Treffer behalten
                    let mut installed = Vec::new();

                    for result in results {
                        if let Some(manager) =
                            managers.iter().find(|m| m.name == result.manager)
                        {
                            if manager.is_installed(&result.real_name).await {
                                installed.push(result);
                            }
                        }
                    }

                    if installed.is_empty() {
                        println!("'{}' ist nicht installiert.", pkg);
                        continue;
                    }

                    let best = &installed[0];

                    let manager = managers
                        .iter()
                        .find(|m| m.name == best.manager)
                        .expect("Manager nicht gefunden");

                    if confirm_action(&format!(
                        "'{}' über '{}' aktualisieren?",
                        best.real_name,
                        manager.name
                    )) {
                        manager.update().await;
                        manager.upgrade_package(&best.real_name).await;
                    }
                }
            }
            println!("Update abgeschlossen.");
        }

        // =========================
        // LIST
        // =========================
        "list" | "l" | "-l" | "-list" => {

            let filters: Vec<String> = packages
                .iter()
                .map(|p| p.to_lowercase())
                .collect();

            println!("Installierte Pakete:\n");

            for manager in &managers {

                let packages =
                    manager.list_installed().await;

                if packages.is_empty() {
                    continue;
                }

                println!("=== {} ===", manager.name);

                for pkg in packages {

                    // 🔥 keine Filter → alles anzeigen
                    if filters.is_empty() {
                        println!("{}", pkg);
                        continue;
                    }

                    let pkg_lc = pkg.to_lowercase();

                    // 🔥 Filter anwenden
                    if filters.iter().any(|f| {
                        pkg_lc.contains(f)
                    }) {
                        println!("{}", pkg);
                    }
                }

                println!();
            }
        }

        _ => println!("Unbekannter Befehl"),
    }
}