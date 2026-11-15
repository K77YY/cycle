# cycle

A minimal,  tui interval timer with a japanese UI. Made for Arch Linux and Android.

## Install (Arch Linux)

### Option 1: build a pacman package (recommended)

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
cd packaging && makepkg -sif
```

Uninstall: `sudo pacman -R cycle`

### Option 2: prebuilt binary

Download the latest `cycle-vX.Y.Z-x86_64-linux.tar.gz` from the
[Releases page](https://github.com/K77YY/cycle/releases), then:

```bash
tar xzf cycle-v*-x86_64-linux.tar.gz
install -Dm755 cycle-v*-x86_64-linux/cycle ~/.local/bin/cycle   # or: sudo install -Dm755 ... /usr/local/bin/cycle
cycle
```

Make sure `~/.local/bin` is in your `PATH`.

### Option 3: cargo

```bash
cargo install --git https://github.com/K77YY/cycle
```

### Optional packages

```bash
sudo pacman -S --needed noto-fonts-cjk #japanese font if you have none
sudo pacman -S --needed sound-theme-freedesktop pipewire #end-of-stage sound (or libpulse instead of pipewire)
```

Without these the timer still works.

## Usage

```bash
cycle
```

Times are typed as `25` (minutes) or `1:30` (minutes:seconds).

## Files

```
~/.config/cycle/
├── settings.json
└── sequences/<name>.json
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
