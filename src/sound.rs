//! ステージ終了音。追加クレート不要:
//! pw-play / paplay で freedesktop のサウンドを鳴らし、
//! 使えなければ端末ベル (BEL) にフォールバックする。

use std::{
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

const SOUND_FILES: [&str; 2] = [
    "/usr/share/sounds/freedesktop/stereo/complete.oga",
    "/usr/share/sounds/freedesktop/stereo/bell.oga",
];

pub fn play() {
    thread::spawn(|| {
        if let Some(file) = SOUND_FILES.iter().find(|f| Path::new(f).exists()) {
            for player in ["pw-play", "paplay"] {
                let ok = Command::new(player)
                    .arg(file)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if ok {
                    return;
                }
            }
        }
        let mut out = io::stdout();
        let _ = out.write_all(b"\x07");
        let _ = out.flush();
    });
}
