# pkg - Paketmanager-Übersetzer für Linux

Ein Rust-Programm, das als einheitliche Schnittstelle für verschiedene Linux-Paketmanager dient.

## Unterstützte Paketmanager

- **apt** (Debian/Ubuntu)
- **pacman** (Arch Linux)
- **yay** (AUR-Helper für Arch)
- **flatpak** (Flathub)
- **dnf** (Fedora)
- **zypper** (openSUSE)

## Funktionen

- **Installieren** von Paketen
- **Entfernen** von Paketen
- **Aktualisieren** von Paketen
- **Suchen** nach Paketen

## Argumente

- **install**
- **remove**
- **update** (ohne PAKETNAME)
- **search**
- **version** (ohne PAKETNAME)

## Verwendung

```sh
pkg [OPTIONEN] <PAKETNAME>
