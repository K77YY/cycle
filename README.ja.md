# cycle

Rust + Ratatui 製の、シンプルで美しい **TUI インターバルタイマー**。

- 背景色は一切指定しません。透過・ブラー背景のターミナルでもそのまま透けます。
- 前景色は ANSI の名前付き色 + bold/dim のみ。ライト/ダークどちらのテーマにも馴染みます。
- 設定とシーケンスは `~/.config/cycle/` に JSON で保存されます。

## Arch Linux でのインストール

### 1. 依存パッケージ

```bash
sudo pacman -S --needed rust git base-devel
# 日本語フォント (未導入の場合)
sudo pacman -S --needed noto-fonts-cjk
# 終了音 (任意。どれも無ければ端末ベルを鳴らします)
sudo pacman -S --needed sound-theme-freedesktop pipewire   # pw-play
# または: sudo pacman -S libpulse                           # paplay
```

`rustup` を使っている場合は `rust` の代わりに `rustup default stable` でも構いません。

### 2a. pacman パッケージとして入れる (おすすめ)

```bash
cd cycle/packaging   # 更新: git pull 後 makepkg -sif
makepkg -si
```

### 2b. cargo で入れる

```bash
cd cycle
cargo install --path .        # ~/.cargo/bin/cycle にインストール
# PATH に ~/.cargo/bin が無ければ:  export PATH="$HOME/.cargo/bin:$PATH"
```

### 2c. ビルドだけ

```bash
cargo build --release
./target/release/cycle
```

## 使い方 

```bash
cycle
```

起動すると既定のシーケンス **読書55分 → 休憩5分 → 漢検25分 → 休憩5分 (ループ)** が読み込まれます。
`Space` を押せば開始。`q` で終了。最後に使ったシーケンスと音の設定は自動で保存・復元されます。

## キーバインド

### タイマー画面

| キー | 動作 |
|---|---|
| `Space` | 一時停止 / 再開 |
| `n` / `→` | 次のステージへ |
| `p` / `←` | 前のステージへ |
| `r` | セッション全体をリセット (サイクル1・ステージ1に戻る) |
| `e` | シーケンス編集 |
| `s` | シーケンスを保存 (名前を入力) |
| `l` | シーケンスを読み込み |
| `m` | 音のオン/オフ |
| `h` | メニューの表示/非表示 (カウントダウンと進捗バーのみ表示。設定は保存) |
| `?` | ヘルプ表示 |
| `q` / `Ctrl+C` | 終了 |

### 編集画面 (`e`)

| キー | 動作 |
|---|---|
| `↑` `↓` / `j` `k` | ステージ選択 |
| `a` | ステージを追加 (選択中の次に挿入) |
| `Enter` / `e` | ステージを編集 (名前・時間・種類) |
| `x` / `d` / `Delete` | ステージを削除 |
| `Shift+↑↓` / `J` `K` | 並べ替え |
| `t` | 集中 ⇄ 休憩 |
| `+` / `-` | 時間を ±1分 |
| `o` | ループのオン/オフ |
| `n` | シーケンス名の変更 |
| `Esc` / `q` | 編集を完了して反映 (自動保存) |

ステージ編集フォームでは `Tab` / `↑↓` で項目移動、`Enter` で決定、`Esc` でキャンセル。
時間は `25` (25分) または `1:30` (1分30秒) の形式で入力します。

## 保存場所

```
~/.config/cycle/
├── settings.json          # 音のオン/オフ、最後に使ったシーケンス名
└── sequences/
    └── 勉強.json
```

`$XDG_CONFIG_HOME` が設定されている場合は `$XDG_CONFIG_HOME/cycle/` を使います。
シーケンスのファイルは手で編集・コピー・削除できます:

```json
{
  "name": "勉強",
  "looping": true,
  "stages": [
    { "name": "読書", "duration_secs": 3300, "kind": "focus" },
    { "name": "休憩",       "duration_secs": 300,  "kind": "break" }
  ]
}
```

## 音について

ステージが終わると短い音が鳴ります (`m` でオン/オフ、設定は保存されます)。
追加の Rust クレートは使わず、次の順で試します:

1. `pw-play` (pipewire) で `/usr/share/sounds/freedesktop/stereo/complete.oga` を再生
2. `paplay` (libpulse) で同じファイルを再生
3. どちらも使えなければ端末ベル (BEL)

デスクトップ通知は使いません。

## Android 版

スマホ版 (同じ機能・同じ JSON 形式) は [`android/`](android/README.md) にあります。広告・通信・Google サービスなしで、F-Droid 向けです。

## トラブルシューティング

- **罫線やブロックの幅がずれる**: 日本語環境のターミナルで「East Asian Ambiguous Width を全角にする」設定が
  有効だと、`█` `━` `●` `→` などがずれます。この設定を **オフ (半角)** にしてください。
- **日本語が□になる**: `noto-fonts-cjk` などの日本語フォントを入れ、ターミナルのフォントに指定してください。
- **日本語入力**: 名前入力欄は IME の確定文字を受け取ります。確定後に表示されます。
- **音が鳴らない**: `sound-theme-freedesktop` と `pipewire` (または `libpulse`) を入れるか、
  端末のベル設定を確認してください。

## 開発

```
src/
├── main.rs     # ターミナル初期化とイベントループ
├── model.rs    # Stage / Sequence、時間の整形とパース
├── timer.rs    # タイマーロジック (UI 非依存、テストあり)
├── config.rs   # ~/.config/cycle/ の読み書き
├── sound.rs    # 終了音
├── app.rs      # 状態とキー入力処理
└── ui.rs       # 描画 (前景色と属性のみ)
```

```bash
cargo test
cargo run
```

## ライセンス

MIT
