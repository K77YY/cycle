//! コアデータモデル: Stage / Sequence と時間の整形・パース。

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 1ステージの最大長 (999分)
pub const MAX_SECS: u64 = 999 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StageKind {
    Focus,
    Break,
}

impl StageKind {
    pub fn label(self) -> &'static str {
        match self {
            StageKind::Focus => "集中",
            StageKind::Break => "休憩",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            StageKind::Focus => StageKind::Break,
            StageKind::Break => StageKind::Focus,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage {
    pub name: String,
    pub duration_secs: u64,
    pub kind: StageKind,
}

impl Stage {
    pub fn new(name: impl Into<String>, minutes: u64, kind: StageKind) -> Self {
        Self {
            name: name.into(),
            duration_secs: minutes * 60,
            kind,
        }
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.duration_secs)
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sequence {
    pub name: String,
    pub stages: Vec<Stage>,
    #[serde(default = "default_true")]
    pub looping: bool,
}

impl Sequence {
    /// 既定のシーケンス: 読書55分 → 休憩5分 → 漢検25分 → 休憩5分 (ループ)
    pub fn default_pomodoro() -> Self {
        Self {
            name: "勉強".to_string(),
            stages: vec![
                Stage::new("読書", 55, StageKind::Focus),
                Stage::new("休憩", 5, StageKind::Break),
                Stage::new("漢検", 25, StageKind::Focus),
                Stage::new("休憩", 5, StageKind::Break),
            ],
            looping: true,
        }
    }

    pub fn total_secs(&self) -> u64 {
        self.stages.iter().map(|s| s.duration_secs).sum()
    }

    /// 読み込んだデータを検証・補正する。
    pub fn sanitize(&mut self) -> Result<(), String> {
        if self.stages.is_empty() {
            return Err("ステージが1つもありません".to_string());
        }
        for s in &mut self.stages {
            s.duration_secs = s.duration_secs.clamp(1, MAX_SECS);
        }
        if self.name.trim().is_empty() {
            self.name = "無題".to_string();
        }
        Ok(())
    }
}

/// `mm:ss` または `h:mm:ss` 形式の時計表示
pub fn fmt_clock(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

/// 「1時間30分」「55分」「1分30秒」のような日本語表記
pub fn fmt_human(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let mut out = String::new();
    if h > 0 {
        out.push_str(&format!("{h}時間"));
    }
    if m > 0 {
        out.push_str(&format!("{m}分"));
    }
    if s > 0 {
        out.push_str(&format!("{s}秒"));
    }
    if out.is_empty() {
        out.push_str("0秒");
    }
    out
}

/// 入力欄用の表記 (`25` または `1:30`)
pub fn fmt_input(secs: u64) -> String {
    if secs % 60 == 0 {
        format!("{}", secs / 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

/// 入力された時間をパースする。`25` (分) または `分:秒`。
pub fn parse_duration(input: &str) -> Result<u64, String> {
    const BAD: &str = "時間の形式が正しくありません (例: 25 または 0:30)";
    let t = input.trim();
    if t.is_empty() {
        return Err("時間を入力してください".to_string());
    }
    let secs = match t.split_once(':') {
        None => t
            .parse::<u64>()
            .map_err(|_| BAD.to_string())?
            .saturating_mul(60),
        Some((m, s)) => {
            let m = if m.is_empty() {
                0
            } else {
                m.parse::<u64>().map_err(|_| BAD.to_string())?
            };
            let s = s.parse::<u64>().map_err(|_| BAD.to_string())?;
            if s >= 60 {
                return Err("秒は 0〜59 で入力してください".to_string());
            }
            m.saturating_mul(60) + s
        }
    };
    if secs == 0 {
        return Err("1秒以上にしてください".to_string());
    }
    if secs > MAX_SECS {
        return Err(format!("{}分以内にしてください", MAX_SECS / 60));
    }
    Ok(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_durations() {
        assert_eq!(parse_duration("25"), Ok(1500));
        assert_eq!(parse_duration("0:30"), Ok(30));
        assert_eq!(parse_duration("1:30"), Ok(90));
        assert!(parse_duration("0").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("1:75").is_err());
    }

    #[test]
    fn formats() {
        assert_eq!(fmt_clock(3300), "55:00");
        assert_eq!(fmt_clock(5400), "1:30:00");
        assert_eq!(fmt_human(90), "1分30秒");
        assert_eq!(fmt_human(3300), "55分");
    }

    #[test]
    fn default_sequence_is_55_5_25_5() {
        let s = Sequence::default_pomodoro();
        let mins: Vec<u64> = s.stages.iter().map(|x| x.duration_secs / 60).collect();
        assert_eq!(mins, vec![55, 5, 25, 5]);
    }
}
