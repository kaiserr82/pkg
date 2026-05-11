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

pub fn detect_distro() -> String {
    std::fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .to_lowercase()
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
            "pacman" => Command::new("sudo").args(["pacman", "-Sy", "--noconfirm"]).spawn(),
            "yay" => Command::new("yay").args(["-Sy", "--noconfirm"]).spawn(),
            "flatpak" => Command::new("flatpak").args(["update", "-y"]).spawn(),
            "dnf" => Command::new("sudo").args(["dnf", "upgrade", "-y"]).spawn(),
            "zypper" => Command::new("sudo").args(["zypper", "--non-interactive", "update"]).spawn(),
            _ => return,
        };

        if let Ok(mut c) = cmd {
            let _ = c.wait().await;
        }
    }

    /// Systemupgrade
    pub async fn upgrade(&self) {
        let cmd = match self.name {
            "apt" => Command::new("sudo").args(["apt", "upgrade"]).spawn(),
            "pacman" => Command::new("sudo").args(["pacman", "-Syu"]).spawn(),
            "yay" => Command::new("yay").args(["-Syu"]).spawn(),
            "dnf" => Command::new("sudo").args(["dnf", "upgrade", "-y"]).spawn(),
            "zypper" => Command::new("sudo").args(["zypper", "", "update"]).spawn(),
            _ => return,
        };

        if let Ok(mut c) = cmd {
            let _ = c.wait().await;
        }
    }

    /// Cleanup
    pub async fn cleanup(&self) {
        let cmd = match self.name {
            "apt" => Command::new("sudo").args(["apt", "autoremove"]).spawn(),
            "pacman" => Command::new("sudo").args(["pacman", "-Scc"]).spawn(),
            "yay" => Command::new("yay").args(["-Scc"]).spawn(),
            "dnf" => Command::new("sudo").args(["dnf", "autoremove"]).spawn(),
            "zypper" => Command::new("sudo").args(["zypper", "remove --clean-deps"]).spawn(),
            _ => return,
        };

        if let Ok(mut c) = cmd {
            let _ = c.wait().await;
        }
    }

    /// Aktualisiert ein einzelnes Paket
    pub async fn upgrade_package(&self, pkg: &str) {
        let cmd = match self.name {
            "apt" => Command::new("sudo")
                .args(["apt", "install", "--only-upgrade", "", pkg])
                .spawn(),

            "pacman" => Command::new("sudo")
                .args(["pacman", "-S", "", "--noconfirm", pkg])
                .spawn(),

            "yay" => Command::new("yay")
                .args(["-S", "--needed", "", pkg])
                .spawn(),

            "flatpak" => Command::new("flatpak")
                .args(["update", "", pkg])
                .spawn(),

            "dnf" => Command::new("sudo")
                .args(["dnf", "upgrade", "", pkg])
                .spawn(),

            "zypper" => Command::new("sudo")
                .args(["zypper", "", "update", pkg])
                .spawn(),

            _ => return,
        };

        if let Ok(mut c) = cmd {
            let _ = c.wait().await;
        }
    }


    /// Suche Paket im Manager
    pub async fn search(&self, pkg: &str) -> Vec<UnifiedPackage> {
        // 🔹 alles in lowercase für Vergleich
        let query = pkg.to_lowercase();

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
                "pacman" | "yay" => line
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.split('/').last()),
                "flatpak" => line.split_whitespace().next(),
                "dnf" => line
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.split('.').next()),
                "zypper" => line.split_whitespace().next(),
                _ => None,
            };

            if let Some(name) = name_opt {
                let name_lc = name.to_lowercase();

                // 🔹 Basis Fuzzy Score
                let mut score = fuzzy_score(&query, &name_lc) as i64;

                // 🔥 EXAKT MATCH → sehr hoher Boost
                if name_lc == query {
                    score += 1000;
                }
                // 🔥 beginnt mit Suchbegriff
                else if name_lc.starts_with(&query) {
                    score += 500;
                }
                // 🔥 enthält Suchbegriff
                else if name_lc.contains(&query) {
                    score += 200;
                }

                // optional: leicht bestrafen wenn sehr lang
                score -= name_lc.len() as i64 / 10;

                if score >= 50 {
                    results.push(UnifiedPackage {
                        // Normalisierter Name für Anzeige, Vergleich und Sortierung
                        name: name.to_string(),

                        // Originalname exakt so speichern, wie er vom Paketmanager
                        // geliefert wurde. Dieser Name sollte später für
                        // install(), remove() usw. verwendet werden.
                        real_name: name.to_string(),

                        // Name des Paketmanagers (apt, pacman, yay, ...)
                        manager: self.name.to_string(),

                        // Berechneter Relevanz-Score
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
        text.to_lowercase().eq(&pkg.to_lowercase())
    }

    //Liste pakete auf die installiert sind
    /// Listet installierte Pakete des Managers auf
    pub async fn list_installed(&self) -> Vec<String> {
        let output = match self.name {
            "apt" => Command::new("dpkg")
                .args(["-l"])
                .output()
                .await,

            "pacman" | "yay" => Command::new("pacman")
                .args(["-Q"])
                .output()
                .await,

            "flatpak" => Command::new("flatpak")
                .args(["list", "--app"])
                .output()
                .await,

            "dnf" => Command::new("dnf")
                .args(["list", "installed"])
                .output()
                .await,

            "zypper" => Command::new("zypper")
                .args(["se", "-i"])
                .output()
                .await,

            _ => return vec![],
        };

        let out = match output {
            Ok(o) => o,
            Err(_) => return vec![],
        };

        let text = String::from_utf8_lossy(&out.stdout);

        let mut packages = Vec::new();

        for line in text.lines() {
            let pkg = match self.name {
                // dpkg -l
                "apt" => {
                    if line.starts_with("ii") {
                        line.split_whitespace().nth(1)
                    } else {
                        None
                    }
                }

                // pacman -Q
                "pacman" | "yay" => {
                    line.split_whitespace().next()
                }

                // flatpak
                "flatpak" => {
                    line.split_whitespace().next()
                }

                // dnf
                "dnf" => {
                    line.split_whitespace().next()
                        .and_then(|s| s.split('.').next())
                }

                // zypper
                "zypper" => {
                    line.split('|').nth(2).map(str::trim)
                }

                _ => None,
            };

            if let Some(name) = pkg {
                packages.push(name.to_string());
            }
        }

        packages.sort();
        packages.dedup();

        packages
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

pub fn manager_priority(manager: &str) -> i64 {
    let os = detect_distro();

    if os.contains("arch") {
        match manager {
            "pacman" => 1000,
            "yay" => 900,
            "flatpak" => 100,
            _ => 0,
        }
    } else if os.contains("ubuntu") || os.contains("debian") {
        match manager {
            "apt" => 1000,
            "flatpak" => 500,
            _ => 0,
        }
    } else {
        0
    }
}