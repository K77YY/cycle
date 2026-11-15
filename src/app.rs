//! アプリケーション状態とキー入力処理。

use crate::config::{self, Settings};
use crate::model::{fmt_input, parse_duration, Sequence, Stage, StageKind, MAX_SECS};
use crate::sound;
use crate::timer::Timer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

pub struct StageForm {
    /// None = 新規追加
    pub index: Option<usize>,
    pub name: String,
    pub duration: String,
    pub kind: StageKind,
    /// 0 = 名前, 1 = 時間, 2 = 種類
    pub focus: usize,
    pub error: Option<String>,
}

impl StageForm {
    fn new_stage() -> Self {
        Self {
            index: None,
            name: String::new(),
            duration: "25".to_string(),
            kind: StageKind::Focus,
            focus: 0,
            error: None,
        }
    }

    fn from_stage(index: usize, s: &Stage) -> Self {
        Self {
            index: Some(index),
            name: s.name.clone(),
            duration: fmt_input(s.duration_secs),
            kind: s.kind,
            focus: 0,
            error: None,
        }
    }

    fn validate(&self) -> Result<Stage, String> {
        let secs = parse_duration(&self.duration)?;
        let name = self.name.trim();
        Ok(Stage {
            name: if name.is_empty() {
                self.kind.label().to_string()
            } else {
                name.to_string()
            },
            duration_secs: secs,
            kind: self.kind,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    Save,
    Rename,
}

pub struct Prompt {
    pub kind: PromptKind,
    pub text: String,
    pub error: Option<String>,
}

pub struct LoadList {
    pub items: Vec<Sequence>,
    pub selected: usize,
}

pub enum Mode {
    Timer,
    /// bool = 編集画面から開いたか
    Help(bool),
    Editor,
    StageForm(StageForm),
    Prompt(Prompt),
    Load(LoadList),
}

pub struct App {
    pub timer: Timer,
    pub settings: Settings,
    pub mode: Mode,
    /// 編集中のシーケンス (編集画面を閉じたときに反映)
    pub draft: Sequence,
    pub editor_selected: usize,
    pub flash: Option<(String, Instant)>,
    pub should_quit: bool,
}

fn back(editor: bool) -> Mode {
    if editor {
        Mode::Editor
    } else {
        Mode::Timer
    }
}

fn plain_char(key: &KeyEvent) -> bool {
    !key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

impl App {
    pub fn new() -> Self {
        let settings = config::load_settings();
        let seq = settings
            .last_sequence
            .as_deref()
            .and_then(|n| config::load_sequence(n).ok())
            .unwrap_or_else(Sequence::default_pomodoro);
        Self {
            draft: seq.clone(),
            timer: Timer::new(seq),
            settings,
            mode: Mode::Timer,
            editor_selected: 0,
            flash: None,
            should_quit: false,
        }
    }

    pub fn in_editor(&self) -> bool {
        match &self.mode {
            Mode::Editor | Mode::StageForm(_) => true,
            Mode::Prompt(p) => p.kind == PromptKind::Rename,
            Mode::Help(e) => *e,
            _ => false,
        }
    }

    pub fn flash_text(&self) -> Option<&str> {
        self.flash.as_ref().map(|(m, _)| m.as_str())
    }

    pub fn notify(&mut self, msg: impl Into<String>) {
        self.flash = Some((msg.into(), Instant::now()));
    }

    /// 毎フレーム呼ぶ: タイマー更新・通知の期限切れ処理
    pub fn tick(&mut self) {
        if self.timer.tick(Instant::now()) && self.settings.sound {
            sound::play();
        }
        if let Some((_, t)) = &self.flash {
            if t.elapsed() > Duration::from_secs(3) {
                self.flash = None;
            }
        }
    }

    // ---- 永続化 ----

    fn try_persist(&mut self) -> Result<(), String> {
        let seq = self.timer.sequence().clone();
        config::save_sequence(&seq)?;
        self.settings.last_sequence = Some(seq.name);
        config::save_settings(&self.settings)
    }

    fn persist(&mut self) {
        if let Err(e) = self.try_persist() {
            self.notify(format!("保存に失敗しました: {e}"));
        }
    }

    pub fn persist_on_exit(&mut self) {
        if let Err(e) = self.try_persist() {
            eprintln!("cycle: 保存に失敗しました: {e}");
        }
    }

    // ---- キー入力 ----

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        let mode = std::mem::replace(&mut self.mode, Mode::Timer);
        self.mode = match mode {
            Mode::Timer => self.key_timer(key),
            Mode::Help(editor) => back(editor),
            Mode::Editor => self.key_editor(key),
            Mode::StageForm(f) => self.key_form(f, key),
            Mode::Prompt(p) => self.key_prompt(p, key),
            Mode::Load(l) => self.key_load(l, key),
        };
    }

    fn key_timer(&mut self, key: KeyEvent) -> Mode {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(' ') => self.timer.toggle(),
            KeyCode::Char('n') | KeyCode::Right => self.timer.next(),
            KeyCode::Char('p') | KeyCode::Left => self.timer.prev(),
            KeyCode::Char('r') => self.timer.reset(),
            KeyCode::Char('e') => {
                self.draft = self.timer.sequence().clone();
                self.editor_selected = self.timer.index().min(self.draft.stages.len() - 1);
                return Mode::Editor;
            }
            KeyCode::Char('s') => {
                return Mode::Prompt(Prompt {
                    kind: PromptKind::Save,
                    text: self.timer.sequence().name.clone(),
                    error: None,
                });
            }
            KeyCode::Char('l') => return self.open_load(),
            KeyCode::Char('m') => self.toggle_sound(),
            KeyCode::Char('h') => self.toggle_menu(),
            KeyCode::Char('?') => return Mode::Help(false),
            _ => {}
        }
        Mode::Timer
    }

    fn toggle_sound(&mut self) {
        self.settings.sound = !self.settings.sound;
        if let Err(e) = config::save_settings(&self.settings) {
            self.notify(format!("設定の保存に失敗しました: {e}"));
            return;
        }
        if self.settings.sound {
            sound::play();
            self.notify("音: オン");
        } else {
            self.notify("音: オフ");
        }
    }

    fn toggle_menu(&mut self) {
        self.settings.show_menu = !self.settings.show_menu;
        if let Err(e) = config::save_settings(&self.settings) {
            self.notify(format!("設定の保存に失敗しました: {e}"));
            return;
        }
        if self.settings.show_menu {
            self.notify("メニュー: 表示");
        } else {
            self.notify("メニュー: 非表示 (h で再表示)");
        }
    }

    fn open_load(&mut self) -> Mode {
        // 現在のシーケンスも一覧に出るよう先に保存しておく
        self.persist();
        match config::list_sequences() {
            Ok(items) if !items.is_empty() => {
                let current = &self.timer.sequence().name;
                let selected = items.iter().position(|s| &s.name == current).unwrap_or(0);
                Mode::Load(LoadList { items, selected })
            }
            Ok(_) => {
                self.notify("保存されたシーケンスがありません");
                Mode::Timer
            }
            Err(e) => {
                self.notify(format!("読み込みに失敗しました: {e}"));
                Mode::Timer
            }
        }
    }

    fn key_load(&mut self, mut l: LoadList, key: KeyEvent) -> Mode {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('l') => return Mode::Timer,
            KeyCode::Up | KeyCode::Char('k') => l.selected = l.selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                if l.selected + 1 < l.items.len() {
                    l.selected += 1;
                }
            }
            KeyCode::Enter => {
                let seq = l.items[l.selected].clone();
                let name = seq.name.clone();
                self.timer.set_sequence(seq);
                self.settings.last_sequence = Some(name.clone());
                match config::save_settings(&self.settings) {
                    Ok(()) => self.notify(format!("読み込みました: {name}")),
                    Err(e) => self.notify(format!("設定の保存に失敗しました: {e}")),
                }
                return Mode::Timer;
            }
            _ => {}
        }
        Mode::Load(l)
    }

    // ---- 編集画面 ----

    fn key_editor(&mut self, key: KeyEvent) -> Mode {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let len = self.draft.stages.len();
        match key.code {
            KeyCode::Up if shift => self.move_stage(-1),
            KeyCode::Down if shift => self.move_stage(1),
            KeyCode::Char('K') => self.move_stage(-1),
            KeyCode::Char('J') => self.move_stage(1),
            KeyCode::Up | KeyCode::Char('k') => {
                self.editor_selected = self.editor_selected.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.editor_selected + 1 < len {
                    self.editor_selected += 1;
                }
            }
            KeyCode::Char('a') => return Mode::StageForm(StageForm::new_stage()),
            KeyCode::Enter | KeyCode::Char('e') => {
                let i = self.editor_selected;
                return Mode::StageForm(StageForm::from_stage(i, &self.draft.stages[i]));
            }
            KeyCode::Char('x') | KeyCode::Char('d') | KeyCode::Delete => self.delete_stage(),
            KeyCode::Char('t') => {
                let s = &mut self.draft.stages[self.editor_selected];
                s.kind = s.kind.toggled();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => self.adjust_duration(60),
            KeyCode::Char('-') => self.adjust_duration(-60),
            KeyCode::Char('o') => self.draft.looping = !self.draft.looping,
            KeyCode::Char('n') => {
                return Mode::Prompt(Prompt {
                    kind: PromptKind::Rename,
                    text: self.draft.name.clone(),
                    error: None,
                });
            }
            KeyCode::Char('?') => return Mode::Help(true),
            KeyCode::Esc | KeyCode::Char('q') => {
                self.apply_draft();
                return Mode::Timer;
            }
            _ => {}
        }
        Mode::Editor
    }

    fn move_stage(&mut self, delta: isize) {
        let i = self.editor_selected;
        let j = i as isize + delta;
        if j < 0 || j as usize >= self.draft.stages.len() {
            return;
        }
        self.draft.stages.swap(i, j as usize);
        self.editor_selected = j as usize;
    }

    fn delete_stage(&mut self) {
        if self.draft.stages.len() <= 1 {
            self.notify("最後のステージは削除できません");
            return;
        }
        self.draft.stages.remove(self.editor_selected);
        self.editor_selected = self.editor_selected.min(self.draft.stages.len() - 1);
    }

    fn adjust_duration(&mut self, delta: i64) {
        let s = &mut self.draft.stages[self.editor_selected];
        s.duration_secs = (s.duration_secs as i64 + delta).clamp(1, MAX_SECS as i64) as u64;
    }

    fn apply_draft(&mut self) {
        let cur = self.timer.sequence();
        if self.draft == *cur {
            return;
        }
        if self.draft.stages == cur.stages {
            // 名前・ループ設定のみの変更: 進行状況は保つ
            self.timer
                .set_meta(self.draft.name.clone(), self.draft.looping);
        } else {
            self.timer.set_sequence(self.draft.clone());
        }
        self.persist();
        self.notify("シーケンスを更新しました");
    }

    fn key_form(&mut self, mut f: StageForm, key: KeyEvent) -> Mode {
        f.error = None;
        match key.code {
            KeyCode::Esc => return Mode::Editor,
            KeyCode::Tab | KeyCode::Down => f.focus = (f.focus + 1) % 3,
            KeyCode::BackTab | KeyCode::Up => f.focus = (f.focus + 2) % 3,
            KeyCode::Enter => match f.validate() {
                Ok(stage) => {
                    match f.index {
                        Some(i) => self.draft.stages[i] = stage,
                        None => {
                            let pos = (self.editor_selected + 1).min(self.draft.stages.len());
                            self.draft.stages.insert(pos, stage);
                            self.editor_selected = pos;
                        }
                    }
                    return Mode::Editor;
                }
                Err(e) => f.error = Some(e),
            },
            KeyCode::Backspace => match f.focus {
                0 => {
                    f.name.pop();
                }
                1 => {
                    f.duration.pop();
                }
                _ => {}
            },
            KeyCode::Left | KeyCode::Right | KeyCode::Char(' ') if f.focus == 2 => {
                f.kind = f.kind.toggled()
            }
            KeyCode::Char(c) if plain_char(&key) => match f.focus {
                0 => {
                    if f.name.chars().count() < 30 {
                        f.name.push(c);
                    }
                }
                1 => {
                    if c.is_ascii_digit() || c == ':' {
                        f.duration.push(c);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Mode::StageForm(f)
    }

    // ---- 名前入力 (保存 / 名前変更) ----

    fn key_prompt(&mut self, mut p: Prompt, key: KeyEvent) -> Mode {
        let editor = p.kind == PromptKind::Rename;
        p.error = None;
        match key.code {
            KeyCode::Esc => return back(editor),
            KeyCode::Enter => {
                let name = p.text.trim().to_string();
                if name.is_empty() {
                    p.error = Some("名前を入力してください".to_string());
                } else {
                    match p.kind {
                        PromptKind::Save => {
                            self.timer.rename(name.clone());
                            match self.try_persist() {
                                Ok(()) => self.notify(format!("保存しました: {name}")),
                                Err(e) => self.notify(format!("保存に失敗しました: {e}")),
                            }
                        }
                        PromptKind::Rename => self.draft.name = name,
                    }
                    return back(editor);
                }
            }
            KeyCode::Backspace => {
                p.text.pop();
            }
            KeyCode::Char(c) if plain_char(&key) => {
                if p.text.chars().count() < 40 {
                    p.text.push(c);
                }
            }
            _ => {}
        }
        Mode::Prompt(p)
    }
}
