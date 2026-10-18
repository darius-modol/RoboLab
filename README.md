# RoboLab

RoboLab ist das Robotik-Praktikum der TU Dresden. Ein LEGO Mindstorms EV3 Roboter soll ein Labyrinth aus Linien erkunden, ohne vorher eine Karte zu haben, und dabei mit einem zentralen Prüfungsserver kommunizieren.

Der Bot ist in Rust geschrieben und in vier Crates aufgeteilt:

- `bot` - Hauptschleife, entscheidet an jedem Knoten, welchen Pfad der Roboter als nächstes nimmt
- `movement` - Motorsteuerung, PID-geregelte Linienverfolgung, Odometrie aus den Motor-Encodern
- `communication` - MQTT-Anbindung an den Server: Pfade melden, Zielvorgaben empfangen, Exploration synchron halten
- `planet` - interne Karte der bereits erkundeten Umgebung samt Pfadsuche

Cross-Compilation auf die EV3-Hardware (ARMv5) läuft über Nix und Zig. Entstanden als Gruppenarbeit.

## Setup

### `rustup` installieren

```sh
# debian based (e.g. Debian, Ubuntu, Linux Mint)
apt install rustup
# arch based (e.g. Archlinux)
pacman -S rustup
# rpm based distro (e.g. fedora)
dnf install rustup
# macos & https://brew.sh/
brew install rustup
# windows & powershell
winget install rustlang.rustup
```

Falls `rustup` nicht über den Paketmanager verfügbar ist, siehe die [rustup]-Website.

### `zig` installieren

```sh
# arch based (e.g. Archlinux)
pacman -S zig
# rpm based distro (e.g. fedora)
dnf install zig
# macos
brew install zig
# windows & powershell
winget install zig.zig
```

Falls `zig` nicht über den Paketmanager verfügbar ist, `pipx`/`uv tool` nutzen oder [zig.guide] konsultieren.

### Rust-Toolchain einrichten

```sh
# linux & macOS
rustup install stable
# windows
rustup install stable-gnu
```

### `just` installieren

`just` ist ein Command-Runner, der CLI-Shortcuts bereitstellt.

```sh
cargo install --locked just
```

Weitere Installationsoptionen im [just project].

### Helfer für Cross-Compilation installieren

```sh
just setup
```

### Projekt bauen

```sh
# für den Bot
just build-bot
# für die lokale Architektur
just build-local
```

### Tests ausführen

```sh
just test
```

## Deployment auf den Bot

### SSH & ssh-agent einrichten

Siehe RoboLab-Dokumentation.

### Code bauen und auf den Bot pushen

```sh
just deploy
```

[zig.guide]: https://zig.guide/getting-started/installation
[rustup]: https://rustup.rs
[just project]: https://github.com/casey/just#packages
