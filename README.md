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


## Installation

### Debian/Ubuntu
Download pkg_0.2.0_amd64.deb für x86_64 Prozessoren oder pkg_0.2.0_arm64.deb für arm Prozessoren (z.B.: Raspi).

Installieren mit 

```sh
sudo apk install ./pkg_0.2.0_amd64.deb 
oder
sudo apk install ./pkg_0.2.0_arm64.deb 

### Arch
Download pkg-0.2.0.tar.gz für x86_64 Prozessoren.

Installieren mit 

```sh
sudo pacman -U pkg-0.2.0.tar.gz
oder Repo benutzen:
https://github.com/kaiserr82/dragonos-repo
