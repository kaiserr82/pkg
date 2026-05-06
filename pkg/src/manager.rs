use tokio::process::Command;
use which::which;

use crate::ranking::fuzzy_score;
use crate::search::UnifiedPackage;

/// Repräsentiert einen Paketmanager
#[derive(Debug, Clone)]
pub struct Manager {
    pub name: &'static str,
    pub priority: i32,
}

impl Manager {
    /// Prüft, ob der Paketmanager im System verfügbar ist
    pub fn is_available(&self) -> bool {
        which(self.name).is_ok()
    }

    /// Installiert ein Paket
    pub async fn install(&self, pkg: &str) {
        let cmd = match self.name {
            "apt" => Command::new("sudo").args(["apt", "install", "-y", pkg]).spawn(),
            "pacman" => Command::new("sudo").args(["pacman", "-S", "--noconfirm", pkg]).spawn(),
            "yay" => Command::new("yay").args(["-S", "--noconfirm", pkg]).spawn(),
            "flatpak" => Command::new("flatpak").args(["install", "-y", pkg]).spawn(),
            "dnf" => Command::new("sudo").args(["dnf", "install", "-y", pkg]).spawn(),
            "zypper" => Command::new("sudo").args(["zypper", "--non-interactive", "install", pkg]).spawn(),
            _ => return,
        };

        if let Ok(mut c) = cmd {
            let _ = c.wait().await;
        }
    }

    /// Entfernt ein Paket
    pub async fn remove(&self, pkg: &str) {
        let cmd = match self.name {
            "apt" => Command::new("sudo").args(["apt", "remove", "-y", pkg]).spawn(),
            "pacman" => Command::new("sudo").args(["pacman", "-R", pkg]).spawn(),
            "yay" => Command::new("yay").args(["-R", pkg]).spawn(),
            "flatpak" => Command::new("flatpak").args(["uninstall", "-y", pkg]).spawn(),
            "dnf" => Command::new("sudo").args(["dnf", "remove", "-y", pkg]).spawn(),
            "zypper" => Command::new("sudo").args(["zypper", "--non-interactive", "remove", pkg]).spawn(),
            _ => return,
        };

        if let Ok(mut c) = cmd {
            let _ = c.wait().await;
        }
    }

    /// Systemupdate
    pub async fn update(&self) {
        let cmd = match self.name {
            "apt" => Command::new("sudo").args(["apt", "update"]).spawn(),
            "pacman" => Command::new("sudo").args(["pacman", "-Syu", "--noconfirm"]).spawn(),
            "yay" => Command::new("yay").args(["-Syu", "--noconfirm"]).spawn(),
            "flatpak" => Command::new("flatpak").args(["update", "-y"]).spawn(),
            "dnf" => Command::new("sudo").args(["dnf", "upgrade", "-y"]).spawn(),
            "zypper" => Command::new("sudo").args(["zypper", "--non-interactive", "update"]).spawn(),
            _ => return,
        };

        if let Ok(mut c) = cmd {
            let _ = c.wait().await;
        }
    }

    /// Suche Paket im Manager
    pub async fn search(&self, pkg: &str) -> Vec<UnifiedPackage> {
        let output = match self.name {
            "apt" => Command::new("apt").args(["search", pkg]).output().await,
            "pacman" => Command::new("pacman").args(["-Ss", pkg]).output().await,
            "yay" => Command::new("yay").args(["-Ss", pkg]).output().await,
            "flatpak" => Command::new("flatpak").args(["search", pkg]).output().await,
            "dnf" => Command::new("dnf").args(["search", pkg]).output().await,
            "zypper" => Command::new("zypper").args(["search", pkg]).output().await,
            _ => return vec![],
        };

        let out = match output {
            Ok(o) => o,
            Err(_) => return vec![],
        };

        let text = String::from_utf8_lossy(&out.stdout);
        let mut results = Vec::new();

        for line in text.lines() {
            let name_opt = match self.name {
                "apt" => line.split('/').next(),
                "pacman" | "yay" => line.split_whitespace().next().and_then(|s| s.split('/').last()),
                "flatpak" => line.split_whitespace().next(),
                "dnf" => line.split_whitespace().next().and_then(|s| s.split('.').next()),
                "zypper" => line.split_whitespace().next(),
                _ => None,
            };

            if let Some(name) = name_opt {
                let score = fuzzy_score(pkg, name) as i64;

                if score >= 50 {
                    results.push(UnifiedPackage {
                        name: name.to_string(),
                        manager: self.name.to_string(),
                        score,
                    });
                }
            }
        }

        results
    }

    /// Prüft ob Paket installiert ist
    pub async fn is_installed(&self, pkg: &str) -> bool {
        let output = match self.name {
            "apt" => Command::new("dpkg").args(["-l", pkg]).output().await,
            "pacman" => Command::new("pacman").args(["-Q", pkg]).output().await,
            "yay" => Command::new("pacman").args(["-Q", pkg]).output().await,
            "flatpak" => Command::new("flatpak").args(["list"]).output().await,
            "dnf" => Command::new("dnf").args(["list", "installed", pkg]).output().await,
            "zypper" => Command::new("zypper").args(["se", "-i", pkg]).output().await,
            _ => return false,
        };

        let out = match output {
            Ok(o) => o,
            Err(_) => return false,
        };

        let text = String::from_utf8_lossy(&out.stdout);
        text.to_lowercase().contains(&pkg.to_lowercase())
    }
}

/// Gibt alle verfügbaren Paketmanager sortiert nach Priorität zurück
pub fn get_managers() -> Vec<Manager> {
    let mut managers = vec![
        Manager { name: "apt", priority: 100 },
        Manager { name: "pacman", priority: 90 },
        Manager { name: "yay", priority: 80 },
        Manager { name: "dnf", priority: 70 },
        Manager { name: "zypper", priority: 60 },
        Manager { name: "flatpak", priority: 50 },
    ];

    managers.sort_by(|a, b| b.priority.cmp(&a.priority));

    managers.into_iter()
        .filter(|m| m.is_available())
        .collect()
}