//! 描画。背景色は一切指定せず、前景色と属性(bold/dim 等)のみを使う。
//! 色は端末のパレット(ANSI 色)に従うので、ライト/ダークどちらのテーマでも馴染む。

use crate::app::{App, LoadList, Mode, Prompt, StageForm};
use crate::model::{fmt_clock, fmt_human, StageKind};
use crate::timer::TimerState;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Wrap,
    },
    Frame,
};

// ---------- スタイル ----------

fn dim() -> Style {
    Style::new().add_modifier(Modifier::DIM)
}
fn bold() -> Style {
    Style::new().add_modifier(Modifier::BOLD)
}
fn reversed() -> Style {
    Style::new().add_modifier(Modifier::REVERSED)
}
fn kind_color(kind: StageKind) -> Color {
    match kind {
        StageKind::Focus => Color::Red,
        StageKind::Break => Color::Green,
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    if app.in_editor() {
        draw_editor(f, app, area);
    } else {
        draw_timer(f, app, area);
    }
    match &app.mode {
        Mode::Help(_) => draw_help(f, area),
        Mode::StageForm(form) => draw_form(f, form, area),
        Mode::Prompt(p) => draw_prompt(f, p, area),
        Mode::Load(l) => draw_load(f, l, area),
        _ => {}
    }
}

// ---------- メイン画面 ----------

fn draw_timer(f: &mut Frame, app: &App, area: Rect) {
    if !app.settings.show_menu {
        draw_zen(f, app, area);
        return;
    }
    let t = &app.timer;
    let seq = t.sequence();
    let area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    if area.width < 24 || area.height < 6 {
        f.render_widget(
            Paragraph::new("ターミナルが小さすぎます").alignment(Alignment::Center),
            area,
        );
        return;
    }

    let stage = t.stage();
    let color = kind_color(stage.kind);
    let big = area.height >= 18;

    let rows = Layout::vertical([
        Constraint::Length(1),                         // 0 ヘッダー
        Constraint::Fill(1),                           // 1
        Constraint::Length(1),                         // 2 ステージ名
        Constraint::Length(if big { 1 } else { 0 }),   // 3
        Constraint::Length(if big { 5 } else { 1 }),   // 4 カウントダウン
        Constraint::Length(if big { 1 } else { 0 }),   // 5
        Constraint::Length(1),                         // 6 プログレスバー
        Constraint::Length(1),                         // 7
        Constraint::Length(1),                         // 8 情報
        Constraint::Length(1),                         // 9 状態
        Constraint::Length(1),                         // 10
        Constraint::Length(2),                         // 11 ステージ一覧
        Constraint::Fill(1),                           // 12
        Constraint::Length(1),                         // 13 フッター
    ])
    .split(area);

    // ヘッダー
    let left = Line::from(vec![
        Span::styled("cycle", bold()),
        Span::styled(format!("  ・  {}", seq.name), dim()),
    ]);
    f.render_widget(Paragraph::new(left), rows[0]);
    let right = format!(
        "ループ {}  ・  音 {}",
        if seq.looping { "オン" } else { "オフ" },
        if app.settings.sound { "オン" } else { "オフ" }
    );
    f.render_widget(
        Paragraph::new(Span::styled(right, dim())).alignment(Alignment::Right),
        rows[0],
    );

    // ステージ名 + 種類
    let label = Line::from(vec![
        Span::styled("● ", Style::new().fg(color)),
        Span::styled(stage.kind.label(), Style::new().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled("  ・  ", dim()),
        Span::styled(stage.name.clone(), bold()),
    ]);
    f.render_widget(Paragraph::new(label).alignment(Alignment::Center), rows[2]);

    // カウントダウン (残りを切り上げて表示)
    let rem_ms = t.remaining_stage().as_millis() as u64;
    let clock = fmt_clock(rem_ms.div_ceil(1000));
    let mut digit_style = Style::new().fg(color).add_modifier(Modifier::BOLD);
    if matches!(t.state(), TimerState::Paused | TimerState::Ready) {
        digit_style = digit_style.add_modifier(Modifier::DIM);
    }
    if big {
        f.render_widget(
            Paragraph::new(big_clock(&clock, rows[4].width))
                .alignment(Alignment::Center)
                .style(digit_style),
            rows[4],
        );
    } else {
        f.render_widget(
            Paragraph::new(Span::styled(clock, digit_style)).alignment(Alignment::Center),
            rows[4],
        );
    }

    // プログレスバー
    let bar = progress_line(t.progress(), color, rows[6].width);
    f.render_widget(Paragraph::new(bar).alignment(Alignment::Center), rows[6]);

    // 情報
    let remaining_label = if seq.looping { "サイクル残り" } else { "全体の残り" };
    let info = Line::from(vec![
        Span::styled("サイクル ", dim()),
        Span::styled(t.cycle().to_string(), bold()),
        Span::styled("    ステージ ", dim()),
        Span::styled(format!("{}/{}", t.index() + 1, seq.stages.len()), bold()),
        Span::styled(format!("    {remaining_label} "), dim()),
        Span::styled(fmt_clock(t.remaining_total().as_secs()), bold()),
    ]);
    f.render_widget(Paragraph::new(info).alignment(Alignment::Center), rows[8]);

    // 状態
    let status = match t.state() {
        TimerState::Ready => Span::styled("Space で開始", dim()),
        TimerState::Running => Span::styled("実行中", Style::new().fg(color)),
        TimerState::Paused => Span::styled("一時停止中", Style::new().fg(Color::Yellow)),
        TimerState::Finished => {
            Span::styled("完了しました ー Space でもう一度", Style::new().fg(Color::Yellow))
        }
    };
    f.render_widget(Paragraph::new(status).alignment(Alignment::Center), rows[9]);

    // ステージ一覧
    let mut spans = Vec::new();
    for (i, s) in seq.stages.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" → ", dim()));
        }
        let text = format!("{} {}", fmt_human(s.duration_secs), s.kind.label());
        if i == t.index() {
            spans.push(Span::styled(
                text,
                Style::new()
                    .fg(kind_color(s.kind))
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
        } else {
            spans.push(Span::styled(text, dim()));
        }
    }
    if seq.looping {
        spans.push(Span::styled(" → (繰り返し)", dim()));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        rows[11],
    );

    // フッター: 通知 or キーヒント
    let footer = if let Some(msg) = app.flash_text() {
        Line::from(Span::styled(msg.to_string(), Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
    } else {
        let space = match t.state() {
            TimerState::Running => "一時停止",
            TimerState::Paused => "再開",
            _ => "開始",
        };
        hints(&[
            ("Space", space),
            ("n", "次へ"),
            ("r", "リセット"),
            ("e", "編集"),
            ("s", "保存"),
            ("l", "読込"),
            ("m", "音"),
            ("h", "メニュー"),
            ("?", "ヘルプ"),
            ("q", "終了"),
        ])
    };
    f.render_widget(
        Paragraph::new(footer).alignment(Alignment::Center).wrap(Wrap { trim: true }),
        rows[13],
    );
}

fn progress_line(p: f64, color: Color, width: u16) -> Line<'static> {
    let w = width.min(64) as usize;
    let bar_w = w.saturating_sub(5).max(1);
    let filled = ((p * bar_w as f64).round() as usize).min(bar_w);
    Line::from(vec![
        Span::styled("━".repeat(filled), Style::new().fg(color)),
        Span::styled("─".repeat(bar_w - filled), dim()),
        Span::styled(format!(" {:>3}%", (p * 100.0).floor() as u32), dim()),
    ])
}

/// メニュー非表示: カウントダウンとプログレスバーだけを中央に表示する。
fn draw_zen(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.timer;
    if area.width < 8 || area.height < 1 {
        return;
    }
    let mut style = Style::new()
        .fg(kind_color(t.stage().kind))
        .add_modifier(Modifier::BOLD);
    if matches!(t.state(), TimerState::Paused | TimerState::Ready) {
        style = style.add_modifier(Modifier::DIM);
    }
    let clock = fmt_clock((t.remaining_stage().as_millis() as u64).div_ceil(1000));
    let big = area.height >= 10 && area.width >= 24;
    let rows = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(if big { 5 } else { 1 }), // カウントダウン
        Constraint::Length(if big { 2 } else { 1 }), // 余白
        Constraint::Length(1),                       // プログレスバー
        Constraint::Fill(1),
    ])
    .split(area);
    if big {
        f.render_widget(
            Paragraph::new(big_clock(&clock, rows[1].width))
                .alignment(Alignment::Center)
                .style(style),
            rows[1],
        );
    } else {
        f.render_widget(
            Paragraph::new(Span::styled(clock, style)).alignment(Alignment::Center),
            rows[1],
        );
    }
    // プログレスバーはメニュー非表示でも残す
    let bar = progress_line(t.progress(), kind_color(t.stage().kind), rows[3].width);
    f.render_widget(Paragraph::new(bar).alignment(Alignment::Center), rows[3]);

    // 切り替え直後などの短い通知だけは下端に出す
    if let Some(msg) = app.flash_text() {
        let r = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        f.render_widget(
            Paragraph::new(Span::styled(
                msg.to_string(),
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            r,
        );
    }
}

fn hints(items: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (k, d) in items {
        spans.push(Span::styled(k.to_string(), bold()));
        spans.push(Span::styled(format!(" {d}  "), dim()));
    }
    Line::from(spans)
}

// ---------- 大きな時計 ----------

fn glyph(c: char) -> [&'static str; 5] {
    match c {
        '0' => ["███", "█ █", "█ █", "█ █", "███"],
        '1' => ["██ ", " █ ", " █ ", " █ ", "███"],
        '2' => ["███", "  █", "███", "█  ", "███"],
        '3' => ["███", "  █", "███", "  █", "███"],
        '4' => ["█ █", "█ █", "███", "  █", "  █"],
        '5' => ["███", "█  ", "███", "  █", "███"],
        '6' => ["███", "█  ", "███", "█ █", "███"],
        '7' => ["███", "  █", "  █", "  █", "  █"],
        '8' => ["███", "█ █", "███", "█ █", "███"],
        '9' => ["███", "█ █", "███", "  █", "███"],
        ':' => [" ", "█", " ", "█", " "],
        _ => ["   ", "   ", "   ", "   ", "   "],
    }
}

fn big_clock(clock: &str, width: u16) -> Vec<Line<'static>> {
    let glyphs: Vec<_> = clock.chars().map(glyph).collect();
    let px: usize = glyphs.iter().map(|g| g[0].chars().count()).sum::<usize>()
        + glyphs.len().saturating_sub(1);
    let scale = if px * 2 <= width as usize { 2 } else { 1 };
    (0..5)
        .map(|r| {
            let mut s = String::new();
            for (i, g) in glyphs.iter().enumerate() {
                if i > 0 {
                    s.push_str(&" ".repeat(scale));
                }
                for ch in g[r].chars() {
                    s.push_str(&(if ch == '█' { "█" } else { " " }).repeat(scale));
                }
            }
            Line::from(s)
        })
        .collect()
}

// ---------- 編集画面 ----------

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(dim())
        .title(Line::from(Span::styled(" シーケンス編集 ", bold())));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let inner = inner.inner(Margin { horizontal: 1, vertical: 0 });

    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(inner);

    let d = &app.draft;
    let header = Line::from(vec![
        Span::styled("名前 ", dim()),
        Span::styled(d.name.clone(), bold()),
        Span::styled("    ループ ", dim()),
        Span::styled(if d.looping { "オン" } else { "オフ" }, bold()),
        Span::styled("    合計 ", dim()),
        Span::styled(fmt_human(d.total_secs()), bold()),
    ]);
    f.render_widget(Paragraph::new(header), rows[0]);

    let table_rows: Vec<Row> = d
        .stages
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(Span::styled(s.kind.label(), Style::new().fg(kind_color(s.kind)))),
                Cell::from(s.name.clone()),
                Cell::from(Line::from(fmt_human(s.duration_secs)).right_aligned()),
            ])
        })
        .collect();
    let table = Table::new(
        table_rows,
        [
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Fill(1),
            Constraint::Length(14),
        ],
    )
    .header(Row::new(vec!["#", "種類", "名前", "時間"]).style(dim()))
    .row_highlight_style(reversed())
    .highlight_symbol("▸ ")
    .column_spacing(2);
    let mut state = TableState::default().with_selected(Some(app.editor_selected));
    f.render_stateful_widget(table, rows[1], &mut state);

    let flash = app
        .flash_text()
        .map(|m| Span::styled(m.to_string(), Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .unwrap_or_default();
    let footer = vec![
        hints(&[
            ("↑↓", "選択"),
            ("a", "追加"),
            ("Enter", "編集"),
            ("x", "削除"),
            ("t", "種類切替"),
            ("+/-", "時間±1分"),
        ]),
        hints(&[
            ("Shift+↑↓", "並べ替え"),
            ("o", "ループ"),
            ("n", "名前変更"),
            ("?", "ヘルプ"),
            ("Esc", "完了"),
        ]),
        Line::from(flash),
    ];
    f.render_widget(Paragraph::new(footer).alignment(Alignment::Center), rows[2]);
}

// ---------- ポップアップ ----------

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect::new(area.x + (area.width - w) / 2, area.y + (area.height - h) / 2, w, h)
}

fn popup(f: &mut Frame, area: Rect, w: u16, h: u16, title: &str) -> Rect {
    let r = centered(area, w, h);
    f.render_widget(Clear, r);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Line::from(Span::styled(format!(" {title} "), bold())));
    let inner = block.inner(r);
    f.render_widget(block, r);
    inner.inner(Margin { horizontal: 1, vertical: 0 })
}

fn help_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key:<16}"), bold()),
        Span::raw(desc.to_string()),
    ])
}

fn draw_help(f: &mut Frame, area: Rect) {
    let inner = popup(f, area, 60, 32, "ヘルプ");
    let head = |t: &str| Line::from(Span::styled(t.to_string(), Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let lines = vec![
        head("タイマー"),
        help_line("Space", "一時停止 / 再開"),
        help_line("n / →", "次のステージへ"),
        help_line("p / ←", "前のステージへ"),
        help_line("r", "セッション全体をリセット"),
        help_line("e", "シーケンス編集"),
        help_line("s", "シーケンスを保存"),
        help_line("l", "シーケンスを読み込み"),
        help_line("m", "音のオン/オフ"),
        help_line("h", "メニューの表示/非表示"),
        help_line("?", "ヘルプ表示"),
        help_line("q", "終了"),
        Line::raw(""),
        head("編集画面"),
        help_line("↑↓ / j k", "選択"),
        help_line("a", "ステージを追加"),
        help_line("Enter / e", "ステージを編集"),
        help_line("x / Delete", "ステージを削除"),
        help_line("Shift+↑↓ / J K", "並べ替え"),
        help_line("t", "集中 / 休憩 を切り替え"),
        help_line("+ / -", "時間を±1分"),
        help_line("o", "ループのオン/オフ"),
        help_line("n", "シーケンス名の変更"),
        help_line("Esc", "編集を完了して戻る"),
        Line::raw(""),
        Line::from(Span::styled("任意のキーで閉じる", dim())),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}

fn field_line(focus: bool, label: &str, value: Vec<Span<'static>>) -> Line<'static> {
    let mut spans = vec![
        Span::styled(if focus { "▸ " } else { "  " }, bold()),
        Span::styled(label.to_string(), if focus { bold() } else { dim() }),
        Span::raw("  "),
    ];
    spans.extend(value);
    Line::from(spans)
}

fn draw_form(f: &mut Frame, form: &StageForm, area: Rect) {
    let title = if form.index.is_some() { "ステージを編集" } else { "ステージを追加" };
    let inner = popup(f, area, 56, 12, title);
    let cursor = |on: bool| Span::styled(if on { "_" } else { "" }, bold());
    let mut lines = vec![
        Line::raw(""),
        field_line(
            form.focus == 0,
            "名前",
            vec![Span::raw(form.name.clone()), cursor(form.focus == 0)],
        ),
        field_line(
            form.focus == 1,
            "時間",
            vec![Span::raw(form.duration.clone()), cursor(form.focus == 1), Span::styled(" 分", dim())],
        ),
        field_line(
            form.focus == 2,
            "種類",
            vec![Span::styled(
                format!("< {} >", form.kind.label()),
                Style::new().fg(kind_color(form.kind)).add_modifier(Modifier::BOLD),
            )],
        ),
        Line::raw(""),
        Line::from(Span::styled("時間は「25」(分) または「1:30」(分:秒)", dim())),
        Line::from(Span::styled("名前が空なら種類名になります", dim())),
    ];
    if let Some(e) = &form.error {
        lines.push(Line::from(Span::styled(e.clone(), Style::new().fg(Color::Red))));
    } else {
        lines.push(Line::raw(""));
    }
    lines.push(hints(&[("Tab", "次の項目"), ("Space/←→", "種類切替"), ("Enter", "決定"), ("Esc", "キャンセル")]));
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn draw_prompt(f: &mut Frame, p: &Prompt, area: Rect) {
    let title = match p.kind {
        crate::app::PromptKind::Save => "シーケンスを保存",
        crate::app::PromptKind::Rename => "シーケンス名の変更",
    };
    let inner = popup(f, area, 54, 8, title);
    let mut lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("名前  ", dim()),
            Span::raw(p.text.clone()),
            Span::styled("_", bold()),
        ]),
        Line::raw(""),
    ];
    if let Some(e) = &p.error {
        lines.push(Line::from(Span::styled(e.clone(), Style::new().fg(Color::Red))));
    } else {
        lines.push(Line::raw(""));
    }
    lines.push(hints(&[("Enter", "決定"), ("Esc", "キャンセル")]));
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_load(f: &mut Frame, l: &LoadList, area: Rect) {
    let h = (l.items.len() as u16 + 5).min(area.height.saturating_sub(2)).max(7);
    let inner = popup(f, area, 64, h, "シーケンスを読み込み");
    let rows = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(inner);
    let items: Vec<ListItem> = l
        .items
        .iter()
        .map(|s| {
            ListItem::new(Line::from(vec![
                Span::raw(s.name.clone()),
                Span::styled(
                    format!("   {}ステージ・{}", s.stages.len(), fmt_human(s.total_secs())),
                    dim(),
                ),
            ]))
        })
        .collect();
    let list = List::new(items).highlight_style(reversed()).highlight_symbol("▸ ");
    let mut state = ListState::default().with_selected(Some(l.selected));
    f.render_stateful_widget(list, rows[0], &mut state);
    f.render_widget(
        Paragraph::new(hints(&[("↑↓", "選択"), ("Enter", "読込"), ("Esc", "戻る")]))
            .alignment(Alignment::Center),
        rows[1],
    );
}
