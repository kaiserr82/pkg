use dialoguer::{theme::ColorfulTheme, Select};
use std::env;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod parser;
mod ranking;
mod manager;
mod language;
mod search;

use language::system_language;
use ranking::{classify_package, type_priority, language_bonus};
use manager::{get_managers};
use search::{unified_search_cached, Cache};

#[tokio::main]
async fn main() {
    let cache: Cache = Arc::new(Mutex::new(HashMap::new()));

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: pkg <install|remove|search|update|version> [package]");
        return;
    }

    let action = &args[1];
    let pkg = args.get(2);

    let managers = get_managers();

    if managers.is_empty() {
        println!("Keine Package Manager gefunden.");
        return;
    }

    match action.to_lowercase().as_str() {

        "version" => {
            let version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
            println!("Version: {}", version);
        }

        "search" => {
            let pkg = pkg.expect("Bitte Paketnamen angeben");

            let results = unified_search_cached(&managers, pkg, cache.clone()).await;

            if results.is_empty() {
                println!("Keine Ergebnisse gefunden.");
                return;
            }

            if results.len() == 1 {
                let r = &results[0];
                if let Some(manager) = managers.iter().find(|m| m.name == r.manager) {
                    manager.install(&r.name).await;
                }
                return;
            }

            let items: Vec<String> = results
                .iter()
                .map(|r| format!("{} ({}) score:{}", r.name, r.manager, r.score))
                .collect();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Paket auswählen (Abbrechen mit STRG + C)")
                .items(&items)
                .default(0)
                .interact_opt()
                .unwrap();

            if let Some(i) = selection {
                let selected = &results[i];

                if let Some(manager) = managers.iter().find(|m| m.name == selected.manager) {
                    manager.install(&selected.name).await;
                }
            }
        }

        "install" => {
            let pkg = pkg.expect("Bitte Paketnamen angeben");

            let mut available = Vec::new();

            for m in &managers {
                if !m.search(pkg).await.is_empty() {
                    available.push(m.clone());
                }
            }

            if available.is_empty() {
                println!("Kein Manager gefunden.");
                return;
            }

            let selected = if available.len() > 1 {
                let options: Vec<&str> = available.iter().map(|m| m.name).collect();

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Installer wählen")
                    .items(&options)
                    .default(0)
                    .interact()
                    .unwrap();

                available[selection].clone()
            } else {
                available[0].clone()
            };

            selected.install(pkg).await;
        }

        "remove" => {
            let pkg = pkg.expect("Bitte Paketnamen angeben");

            let mut installed_on = Vec::new();

            for m in &managers {
                if m.is_installed(pkg).await {
                    installed_on.push(m.clone());
                }
            }

            if installed_on.is_empty() {
                println!("Paket nicht installiert.");
                return;
            }

            let selected = if installed_on.len() > 1 {
                let options: Vec<&str> = installed_on.iter().map(|m| m.name).collect();

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Deinstallationsquelle wählen")
                    .items(&options)
                    .default(0)
                    .interact()
                    .unwrap();

                installed_on[selection].clone()
            } else {
                installed_on[0].clone()
            };

            selected.remove(pkg).await;
        }

        "update" => {
            for m in &managers {
                println!("=== {} ===", m.name);
                m.update().await;
            }
        }

        _ => println!("Unbekannter Befehl"),
    }
}