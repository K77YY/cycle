# cycle

A minimal, good-looking **interval timer for the terminal** (Rust + Ratatui).
Chain focus and break stages into a sequence and let it loop, e.g.
**読書 55 min → 休憩 5 min → 漢検 25 min → 休憩 5 min → repeat**.

The UI is entirely in Japanese. · [日本語 README](README.ja.md)

- Large countdown, progress bar, cycle counter, time left in the cycle
- Custom sequences: add / edit / delete / reorder stages, loop forever
- Save and load multiple sequences; the last one is restored on startup
- Optional end-of-stage sound (toggle with `m`), no desktop notifications
- Foreground colors only: your terminal's transparent / blurred background
  stays visible, and it works on light and dark themes

## Install (Arch Linux)

### Option 1: prebuilt binary (fastest)

Download the latest `cycle-vX.Y.Z-x86_64-linux.tar.gz` from the
[Releases page](https://github.com/K77YY/cycle/releases), then:

```bash
tar xzf cycle-v*-x86_64-linux.tar.gz
install -Dm755 cycle-v*-x86_64-linux/cycle ~/.local/bin/cycle   # or: sudo install -Dm755 ... /usr/local/bin/cycle
cycle
```

Make sure `~/.local/bin` is in your `PATH`.

### Option 2: build a pacman package (recommended)

Installs, upgrades and uninstalls cleanly through pacman.

```bash
sudo pacman -S --needed git base-devel rust
git clone https://github.com/K77YY/cycle.git
cd cycle/packaging
makepkg -si
```

Update later:

```bash
cd cycle && git pull
cd packaging && makepkg -sif      # -f rebuilds and overwrites the existing package
```

Uninstall: `sudo pacman -R cycle`

### Option 3: cargo

```bash
cargo install --git https://github.com/K77YY/cycle
```

### Optional packages

```bash
sudo pacman -S --needed noto-fonts-cjk                 # Japanese font, if you have none
sudo pacman -S --needed sound-theme-freedesktop pipewire   # end-of-stage sound (or libpulse instead of pipewire)
```

Without these the timer still works; the sound falls back to the terminal bell.

## Android app

A phone version with the same features (sequences, loop, sound toggle, hide-menu mode) lives in
[`android/`](android/README.md). It is FOSS-only (no Google services, no network access) and
built for F-Droid. Sequence JSON files are compatible with the terminal version.

## Usage

```bash
cycle
```

The default sequence loads automatically. Press `Space` to start, `q` to quit.

| Key | Action |
|---|---|
| `Space` | Pause / resume (一時停止 / 再開) |
| `n` / `→` | Next stage |
| `p` / `←` | Previous stage |
| `r` | Reset the whole session (cycle 1, stage 1) |
| `e` | Edit sequence |
| `s` | Save sequence (asks for a name) |
| `l` | Load sequence |
| `m` | Sound on / off |
| `h` | Hide / show the menu (only the countdown and progress bar stay; remembered) |
| `?` | Help |
| `q` / `Ctrl+C` | Quit |

### Editor (`e`)

| Key | Action |
|---|---|
| `↑` `↓` / `j` `k` | Select stage |
| `a` | Add stage (inserted after the selection) |
| `Enter` / `e` | Edit stage (name, time, type) |
| `x` / `d` / `Delete` | Delete stage |
| `Shift+↑↓` / `J` `K` | Reorder |
| `t` | Toggle 集中 (focus) ⇄ 休憩 (break) |
| `+` / `-` | Time ±1 minute |
| `o` | Loop on / off |
| `n` | Rename sequence |
| `Esc` / `q` | Finish editing (applied and auto-saved) |

Times are typed as `25` (minutes) or `1:30` (minutes:seconds).

### Example: build the 55 → 5 → 25 → 5 sequence

1. `e` to open the editor.
2. `a` → name `読書`, time `55`, type 集中 → `Enter`
3. `a` → name `休憩`, time `5`, `Tab` `Tab` then `Space` to switch to 休憩 → `Enter`
4. `a` → name `漢検`, time `25`, type 集中 → `Enter`
5. `a` → name `休憩`, time `5`, type 休憩 → `Enter`
6. Check that loop is on (`o` toggles), press `n` to name it, then `Esc`.

## Files

```
~/.config/cycle/
├── settings.json                 # sound on/off, last used sequence
└── sequences/<name>.json         # one file per saved sequence
```

(`$XDG_CONFIG_HOME/cycle/` is used if set.) Sequence files are plain JSON and can be
edited, copied or deleted by hand. To restore the defaults: `rm -rf ~/.config/cycle`.

## Troubleshooting

- **Boxes / bars look misaligned**: turn off "treat East Asian ambiguous-width characters as wide"
  in your terminal's settings.
- **Japanese shows as □**: install `noto-fonts-cjk` and select a CJK-capable font.
- **No sound**: install `sound-theme-freedesktop` plus `pipewire` (or `libpulse`).

## Development

```bash
cargo run
cargo test
```

Layout: `src/model.rs` (data model), `timer.rs` (logic), `config.rs` (persistence),
`sound.rs`, `app.rs` (state and keys), `ui.rs` (rendering).

## License

MIT
