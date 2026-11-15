# GitHub への公開とリリースの手順 / Publishing & releasing

## 1. 初回: リポジトリを作る

```bash
cd ~/cycle

# 1) 一度ビルドして Cargo.lock を作る (バイナリは Cargo.lock をコミットするのが作法。
#    リリースのワークフローも --locked を使うので必須)
cargo build --release

# 2) LICENSE の "YOUR NAME" を自分の名前に直す
$EDITOR LICENSE

# 3) README 内の YOURNAME / packaging/aur/PKGBUILD 内の YOUR_USER を
#    自分の GitHub ユーザー名に置換
sed -i 's/YOURNAME/あなたのユーザー名/g' README.md README.ja.md
sed -i 's/YOUR_USER/あなたのユーザー名/g' packaging/aur/PKGBUILD

# 4) コミット
git init -b main
git add .
git status            # target/ や *.pkg.tar.zst が含まれていないことを確認
git commit -m "Initial commit"
```

**含めるべきファイル**: `Cargo.toml` `Cargo.lock` `src/` `packaging/` `.github/` `README.md` `README.ja.md`
`LICENSE` `RELEASING.md` `.gitignore`。`target/` やビルド済みパッケージは `.gitignore` で除外されます。

### GitHub に push する

**A. `gh` コマンド (簡単)**

```bash
sudo pacman -S --needed github-cli
gh auth login
gh repo create cycle --public --source=. --remote=origin --push
```

**B. ブラウザ**

1. https://github.com/new でリポジトリ `cycle` を作成 (README などの初期ファイルは追加しない)
2. 表示された手順に従って push:

```bash
git remote add origin git@github.com:あなたのユーザー名/cycle.git
git push -u origin main
```

(SSH 鍵が未設定なら https の URL でも可。パスワードの代わりに Personal Access Token が必要です。)

## 2. リリースを作る (ユーザーがダウンロードできる形)

`v` で始まるタグを push すると、`.github/workflows/release.yml` が自動で
静的リンクの x86_64 Linux バイナリをビルドし、Releases に添付します。

```bash
# 1) Cargo.toml と packaging/PKGBUILD の version を上げる (例: 0.2.0)
cargo build --release          # Cargo.lock を更新
git commit -am "Release v0.2.0"
git push

# 2) タグを打って push
git tag v0.2.0
git push origin v0.2.0
```

GitHub の **Actions** タブでビルドを確認し、数分後に **Releases** に
`cycle-v0.2.0-x86_64-linux.tar.gz` と `.sha256` が並びます。

ユーザーのインストール手順:

```bash
tar xzf cycle-v0.2.0-x86_64-linux.tar.gz
install -Dm755 cycle-v0.2.0-x86_64-linux/cycle ~/.local/bin/cycle
```

> ワークフローが失敗する場合: Settings → Actions → General → Workflow permissions を
> "Read and write permissions" にしてください。`Cargo.lock` が未コミットでも失敗します。

### 手動でリリースする場合 (Actions を使わない)

```bash
cargo build --release
tar czf cycle-v0.2.0-x86_64-linux.tar.gz -C target/release cycle
gh release create v0.2.0 cycle-v0.2.0-x86_64-linux.tar.gz --generate-notes
```

(この場合は Arch の glibc に依存したバイナリになります。Arch 以外でも動かしたいなら Actions 版が確実です。)

## 3. 任意: AUR に載せる

`packaging/aur/PKGBUILD` は GitHub のタグ付きソースからビルドする版です。
AUR アカウントを作り、SSH 鍵を登録した上で:

```bash
git clone ssh://aur@aur.archlinux.org/cycle-tui.git aur-cycle-tui
cp packaging/aur/PKGBUILD aur-cycle-tui/
cd aur-cycle-tui
updpkgsums                              # sha256sums を実際の値に更新 (pacman-contrib)
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO && git commit -m "cycle-tui 0.2.0" && git push
```

公開後は `yay -S cycle-tui` のようにインストールできます。
