//! タイマーロジック (UI から独立)。

use crate::model::{Sequence, Stage};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Ready,
    Running,
    Paused,
    Finished,
}

pub struct Timer {
    seq: Sequence,
    index: usize,
    cycle: u32,
    elapsed: Duration,
    state: TimerState,
    last_tick: Instant,
}

impl Timer {
    pub fn new(mut seq: Sequence) -> Self {
        if seq.stages.is_empty() {
            seq = Sequence::default_pomodoro();
        }
        Self {
            seq,
            index: 0,
            cycle: 1,
            elapsed: Duration::ZERO,
            state: TimerState::Ready,
            last_tick: Instant::now(),
        }
    }

    // ---- 取得系 ----
    pub fn sequence(&self) -> &Sequence {
        &self.seq
    }
    pub fn state(&self) -> TimerState {
        self.state
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn cycle(&self) -> u32 {
        self.cycle
    }
    pub fn stage(&self) -> &Stage {
        &self.seq.stages[self.index]
    }

    pub fn remaining_stage(&self) -> Duration {
        self.stage().duration().saturating_sub(self.elapsed)
    }

    pub fn progress(&self) -> f64 {
        let total = self.stage().duration().as_secs_f64();
        (self.elapsed.as_secs_f64() / total).clamp(0.0, 1.0)
    }

    /// 現在のサイクル(=シーケンス1周)の残り時間
    pub fn remaining_total(&self) -> Duration {
        if self.state == TimerState::Finished {
            return Duration::ZERO;
        }
        let later: u64 = self.seq.stages[self.index + 1..]
            .iter()
            .map(|s| s.duration_secs)
            .sum();
        self.remaining_stage() + Duration::from_secs(later)
    }

    // ---- 操作系 ----
    pub fn set_sequence(&mut self, mut seq: Sequence) {
        if seq.stages.is_empty() {
            seq = Sequence::default_pomodoro();
        }
        self.seq = seq;
        self.restart();
    }

    /// 進行状況を保ったまま名前とループ設定だけ変更する。
    pub fn set_meta(&mut self, name: String, looping: bool) {
        self.seq.name = name;
        self.seq.looping = looping;
    }

    pub fn rename(&mut self, name: String) {
        self.seq.name = name;
    }

    fn restart(&mut self) {
        self.index = 0;
        self.cycle = 1;
        self.elapsed = Duration::ZERO;
        self.state = TimerState::Ready;
        self.last_tick = Instant::now();
    }

    pub fn toggle(&mut self) {
        match self.state {
            TimerState::Ready | TimerState::Paused => {
                self.last_tick = Instant::now();
                self.state = TimerState::Running;
            }
            TimerState::Running => {
                self.tick(Instant::now());
                if self.state == TimerState::Running {
                    self.state = TimerState::Paused;
                }
            }
            TimerState::Finished => {
                self.restart();
                self.state = TimerState::Running;
            }
        }
    }

    /// 時間を進める。ステージが切り替わった/終了した場合 true。
    pub fn tick(&mut self, now: Instant) -> bool {
        if self.state != TimerState::Running {
            return false;
        }
        self.elapsed += now.saturating_duration_since(self.last_tick);
        self.last_tick = now;

        let mut ended = false;
        loop {
            let dur = self.stage().duration();
            if self.elapsed < dur {
                break;
            }
            ended = true;
            self.elapsed -= dur;
            if !self.advance() {
                self.state = TimerState::Finished;
                self.elapsed = self.stage().duration();
                break;
            }
        }
        ended
    }

    /// 次のステージへ。進めなければ false (ループなしの最終ステージ)。
    fn advance(&mut self) -> bool {
        if self.index + 1 < self.seq.stages.len() {
            self.index += 1;
            true
        } else if self.seq.looping {
            self.index = 0;
            self.cycle = self.cycle.saturating_add(1);
            true
        } else {
            false
        }
    }

    pub fn next(&mut self) {
        if self.state == TimerState::Finished {
            return;
        }
        self.last_tick = Instant::now();
        self.elapsed = Duration::ZERO;
        if !self.advance() {
            self.state = TimerState::Finished;
            self.elapsed = self.stage().duration();
        }
    }

    pub fn prev(&mut self) {
        self.last_tick = Instant::now();
        self.elapsed = Duration::ZERO;
        if self.state == TimerState::Finished {
            self.state = TimerState::Paused;
            return;
        }
        if self.index > 0 {
            self.index -= 1;
        } else if self.seq.looping && self.cycle > 1 {
            self.index = self.seq.stages.len() - 1;
            self.cycle -= 1;
        }
    }

    /// セッション全体をリセット (サイクル1・ステージ1の待機状態に戻す)
    pub fn reset(&mut self) {
        self.restart();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StageKind;

    fn seq(looping: bool) -> Sequence {
        Sequence {
            name: "t".into(),
            stages: vec![
                Stage::new("a", 1, StageKind::Focus),
                Stage::new("b", 1, StageKind::Break),
            ],
            looping,
        }
    }

    #[test]
    fn advances_and_loops() {
        let mut t = Timer::new(seq(true));
        t.toggle();
        let now = t.last_tick;
        assert!(t.tick(now + Duration::from_secs(60)));
        assert_eq!(t.index(), 1);
        assert!(t.tick(now + Duration::from_secs(120)));
        assert_eq!(t.index(), 0);
        assert_eq!(t.cycle(), 2);
    }

    #[test]
    fn finishes_without_loop() {
        let mut t = Timer::new(seq(false));
        t.toggle();
        let now = t.last_tick;
        assert!(t.tick(now + Duration::from_secs(120)));
        assert_eq!(t.state(), TimerState::Finished);
        assert_eq!(t.remaining_total(), Duration::ZERO);
    }

    #[test]
    fn reset_returns_to_cycle_one_stage_one() {
        let mut t = Timer::new(seq(true));
        t.toggle();
        let now = t.last_tick;
        // 各ステージは60秒: 90秒後 = サイクル1のステージ2
        t.tick(now + Duration::from_secs(90));
        assert_eq!((t.index(), t.cycle()), (1, 1));
        // 150秒後 = サイクル2のステージ1
        t.tick(now + Duration::from_secs(150));
        assert_eq!((t.index(), t.cycle()), (0, 2));
        t.reset();
        assert_eq!((t.index(), t.cycle()), (0, 1));
        assert_eq!(t.state(), TimerState::Ready);
        assert_eq!(t.remaining_total(), Duration::from_secs(120));
    }

    #[test]
    fn remaining_total_sums_later_stages() {
        let t = Timer::new(seq(true));
        assert_eq!(t.remaining_total(), Duration::from_secs(120));
    }
}
