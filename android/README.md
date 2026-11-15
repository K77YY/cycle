# cycle for Android

Terminal 版 **cycle** のスマホ版です。Kotlin + Jetpack Compose、**Google サービス・広告・通信なし**。
F-Droid で配布できる構成です (すべて FOSS の依存のみ)。UI は日本語。

- 大きなカウントダウン / 進捗バー / ステージ名と種類 (集中=赤・休憩=緑) / サイクル数 / サイクル残り時間
- ▶ 一時停止・再開 / ⏮ 前へ / ⏭ 次へ / ⟲ **セッション全体をリセット**
- シーケンス: ステージの追加・編集・削除・並べ替え、ループ、複数保存・読み込み、最後に使ったものを自動復元
- 音のオン/オフ (ステージ終了時のビープ)
- 「隠す」: カウントダウンと進捗バーだけを表示 (タップ = 一時停止/再開、長押し = メニュー再表示)
- 画面を消してもフォアグラウンドサービスで動作継続。ライト/ダーク、Material You (Android 12+) に追従
- シーケンスの JSON は TUI 版と同じ形式 (`sequences/*.json`)。既定は 勉強: 読書55 → 休憩5 → 漢検25 → 休憩5 (ループ)

動作要件: Android 8.0 (API 26) 以上。

## 使う権限

| 権限 | 理由 |
|---|---|
| `FOREGROUND_SERVICE` / `…_SPECIAL_USE` | 画面オフでもタイマーを動かし続けるため |
| `POST_NOTIFICATIONS` | 実行中の常駐通知 (Android の仕様上フォアグラウンドサービスに必須)。拒否してもタイマーは動きます |
| `WAKE_LOCK` | 画面オフ中でもステージ終了音を時間通りに鳴らすため (実行中のみ保持) |

通信権限はありません。

## インストール (ユーザー向け)

- **F-Droid** (審査・公開後): F-Droid アプリで「cycle」を検索してインストール
- **APK を直接**: GitHub の Releases から `cycle-android-v*.apk` をダウンロードして開く
  (「提供元不明のアプリ」の許可が必要)。[Obtainium](https://github.com/ImranR98/Obtainium) に
  リポジトリ URL を登録すると、新しいリリースの通知と更新ができます。

## ビルド (Arch Linux)

```bash
sudo pacman -S --needed jdk17-openjdk gradle git
# Android SDK: Android Studio (AUR: android-studio) の SDK Manager で
#   "Android SDK Platform 35" と "Build-Tools" を入れるのが簡単
export ANDROID_HOME=$HOME/Android/Sdk

# Gradle Wrapper を作る (初回のみ)。
# Arch の gradle は新しすぎて、このプロジェクトの Android Gradle Plugin と合わないことがあるので、
# 空のフォルダで生成してコピーします。
mkdir /tmp/gw && (cd /tmp/gw && gradle wrapper --gradle-version 8.10.2)
cp -r /tmp/gw/gradlew /tmp/gw/gradlew.bat /tmp/gw/gradle android/

cd android
./gradlew testDebugUnitTest assembleDebug
adb install app/build/outputs/apk/debug/app-debug.apk
```

`gradlew` と `gradle/` はコミットしてください (F-Droid / GitHub Actions のビルドに必要)。
Android Studio で `android/` フォルダを開いて ▶ で実行しても構いません。

> バージョン (AGP 8.7.3 / Kotlin 2.0.21 / Compose BOM 2024.12.01) の組み合わせです。
> 新しくしたい場合は `build.gradle.kts` と `app/build.gradle.kts` を更新してください。

### 署名付きリリース APK を作る

```bash
# 1) 署名鍵を作る (一度だけ。紛失すると更新できなくなるのでバックアップ。コミットしない)
keytool -genkeypair -v -keystore ~/cycle-release.jks -alias cycle \
        -keyalg RSA -keysize 4096 -validity 10000

# 2) android/keystore.properties を作る (.gitignore 済み)
printf 'storeFile=%s\nstorePassword=%s\nkeyAlias=cycle\nkeyPassword=%s\n' \
  "$HOME/cycle-release.jks" 'パスワード' 'パスワード' > keystore.properties

# 3) ビルド
./gradlew assembleRelease     # → app/build/outputs/apk/release/app-release.apk
```

## 公開する

1. `app/build.gradle.kts` の `applicationId` (`app.cycle.timer`) を自分のものに変更 (任意。公開前に推奨)
2. `gradlew` / `gradle/` をコミット済みであることを確認
3. **GitHub Releases に APK を載せる (自動)**: リポジトリの Settings → Secrets and variables → Actions に
   `KEYSTORE_BASE64` (`base64 -w0 ~/cycle-release.jks` の出力), `KEYSTORE_PASSWORD`, `KEY_ALIAS`,
   `KEY_PASSWORD` を登録し、タグを push:
   ```bash
   git tag android-v1.0.0 && git push origin android-v1.0.0
   ```
   `.github/workflows/android.yml` がテスト → 署名 → `cycle-android-v1.0.0.apk` を Release に添付します。
   (TUI 版のリリース用タグ `v*` とは別系統なので衝突しません)
4. **F-Droid**: [fdroiddata](https://gitlab.com/fdroid/fdroiddata) を fork し、`android/fdroid/app.cycle.timer.yml` を
   `metadata/app.cycle.timer.yml` として追加して Merge Request を出す (`YOURNAME` を置換)。
   ストア掲載文はリポジトリ直下の `fastlane/metadata/android/` から読まれます。
   F-Droid 側がソースからビルドして署名します (審査には数日〜数週間かかります)。
5. 早く配りたい場合は [IzzyOnDroid](https://apt.izzysoft.de/fdroid/) に GitHub Release の APK を申請できます。

次のリリースは `versionCode` と `versionName` を上げてからタグを打ちます。

## 構成

```
app/src/main/java/app/cycle/timer/
├── Model.kt          # Plan / Stage / Kind、JSON、時間の整形 (TUI 版と互換)
├── TimerCore.kt      # タイマーのロジック (UI/Android 非依存。timer.rs の移植)
├── Engine.kt         # 全体で1つのタイマー + 設定。ティックループと終了音
├── TimerService.kt   # 実行中だけ動くフォアグラウンドサービス (通知・WakeLock)
├── Store.kt          # filesDir への JSON 保存
├── TimerScreen.kt    # メイン画面 / 隠すモード / シーケンス選択シート
├── EditorScreen.kt   # シーケンス編集
├── Theme.kt, Glyphs.kt, MainActivity.kt, CycleApp.kt
app/src/test/…/TimerCoreTest.kt   # JVM 単体テスト (./gradlew testDebugUnitTest)
```
