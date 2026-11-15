# cycle

日本語 UI のミニマルな TUI インターバルタイマー。Arch Linux と Android 向けです。

見た目は使っているターミナルに合わせて変わります。

## インストール (Arch Linux)

### 方法 1: pacman パッケージをビルドする (おすすめ)

pacman でインストール・更新・アンインストールをきれいに管理できます。

```bash
sudo pacman -S --needed git base-devel rust
git clone https://github.com/K77YY/cycle.git
cd cycle/packaging
makepkg -si
```

更新するとき:

```bash
cd cycle && git pull
cd packaging && makepkg -sif      # -f で再ビルドし、既存のパッケージを上書きします
```

アンインストール: `sudo pacman -R cycle`

### 方法 2: ビルド済みバイナリ (いちばん手軽)

[Releases ページ](https://github.com/K77YY/cycle/releases) から最新の
`cycle-vX.Y.Z-x86_64-linux.tar.gz` をダウンロードして、次を実行します:

```bash
tar xzf cycle-v*-x86_64-linux.tar.gz
install -Dm755 cycle-v*-x86_64-linux/cycle ~/.local/bin/cycle   # または: sudo install -Dm755 ... /usr/local/bin/cycle
cycle
```

`~/.local/bin` が `PATH` に入っていることを確認してください。

### 方法 3: cargo

```bash
cargo install --git https://github.com/K77YY/cycle
```

### 追加パッケージ (任意)

```bash
sudo pacman -S --needed noto-fonts-cjk                     # 日本語フォント (未導入の場合)
sudo pacman -S --needed sound-theme-freedesktop pipewire   # ステージ終了音 (pipewire の代わりに libpulse でも可)
```

これらがなくてもタイマーは動きます。その場合、音は端末のベルになります。

## 使い方

```bash
cycle
```

時間は `25` (分) または `1:30` (分:秒) の形式で入力します。

## ファイル

```
~/.config/cycle/
├── settings.json                 # 音のオン/オフ、最後に使ったシーケンス
└── sequences/<名前>.json         # 保存したシーケンスごとに 1 ファイル
```

(`$XDG_CONFIG_HOME` が設定されている場合は `$XDG_CONFIG_HOME/cycle/` を使います。)
シーケンスのファイルはただの JSON なので、手で編集・コピー・削除できます。
初期状態に戻すには: `rm -rf ~/.config/cycle`

## トラブルシューティング

- **罫線やバーの表示がずれる**: ターミナルの設定で「East Asian Ambiguous Width の文字を全角として扱う」を
  オフにしてください。
- **日本語が □ になる**: `noto-fonts-cjk` をインストールし、日本語に対応したフォントを選んでください。
- **音が鳴らない**: `sound-theme-freedesktop` と `pipewire` (または `libpulse`) をインストールしてください。

## 開発

```bash
cargo run
cargo test
```

構成: `src/model.rs` (データモデル)、`timer.rs` (ロジック)、`config.rs` (保存)、
`sound.rs` (音)、`app.rs` (状態とキー入力)、`ui.rs` (描画)。

## ライセンス

MIT
