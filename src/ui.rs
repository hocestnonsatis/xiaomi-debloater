use crate::adb;
use crate::model::{InstalledBloat, Risk};
use anyhow::Result;
use std::borrow::Cow;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub struct App {
    pub items: Vec<InstalledBloat>,
    pub visible_indices: Vec<usize>,
    pub list_state: ListState,
    pub show_advanced: bool,
    pub confirm_remove: bool,
    pub device_serial: Option<String>,
    pub device_info: String,
    pub status: String,
    pub log: Vec<String>,
}

impl App {
    pub fn rebuild_visible(&mut self) {
        self.visible_indices = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, b)| self.show_advanced || b.meta.risk != Risk::Advanced)
            .map(|(i, _)| i)
            .collect();
        let n = self.visible_indices.len();
        if n == 0 {
            self.list_state.select(None);
        } else {
            let cur = self.list_state.selected().unwrap_or(0).min(n - 1);
            self.list_state.select(Some(cur));
        }
    }

    pub fn selected_visible_pkgs(&self) -> Vec<String> {
        let mut v = Vec::new();
        for &idx in &self.visible_indices {
            if self.items[idx].selected {
                v.push(self.items[idx].meta.id.clone());
            }
        }
        v
    }

    pub fn count_selected(&self) -> usize {
        self.visible_indices
            .iter()
            .filter(|&&idx| self.items[idx].selected)
            .count()
    }
}

pub fn draw(frame: &mut Frame<'_>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(if app.log.is_empty() { 2 } else { 6 }),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let title = format!(
        "xiaomi-debloater — {}  [Advanced: {}]",
        app.device_info,
        if app.show_advanced { "shown" } else { "hidden" }
    );
    let header = Paragraph::new(title).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Device"),
    );
    frame.render_widget(header, chunks[0]);

    let list_items: Vec<ListItem> = app
        .visible_indices
        .iter()
        .map(|&idx| {
            let b = &app.items[idx];
            let mark = if b.selected { "[x]" } else { "[ ]" };
            let risk_s = match b.meta.risk {
                Risk::Safe => ("safe", Style::default().fg(Color::Green)),
                Risk::Caution => ("caution", Style::default().fg(Color::Yellow)),
                Risk::Advanced => ("ADV", Style::default().fg(Color::Red)),
            };
            let line = Line::from(vec![
                Span::raw(format!("{mark} ")),
                Span::styled(format!("{:<8} ", risk_s.0), risk_s.1),
                Span::styled(
                    b.meta.id.as_str(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" — "),
                Span::styled(
                    b.meta.category.as_str(),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" — "),
                Span::raw(truncate_desc(&b.meta.description, 50)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    "Installed catalog packages ({}) — {} selected",
                    app.visible_indices.len(),
                    app.count_selected()
                )),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, chunks[1], &mut app.list_state);

    let log_text = if app.log.is_empty() {
        app.status.clone()
    } else {
        app.log.join("\n")
    };
    let log_para = Paragraph::new(log_text)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Log"));
    frame.render_widget(log_para, chunks[2]);

    let help_text: Cow<'static, str> = if app.confirm_remove {
        Cow::Borrowed(
            "CONFIRM:  Y  = remove selected packages   |   N  or  Esc  = cancel",
        )
    } else {
        Cow::Borrowed(
            "↑/↓ move  Space toggle  a select all visible  c clear  A toggle advanced packages  r refresh  x remove  q quit  \
             Uninstall uses: adb shell pm uninstall --user 0 <pkg>",
        )
    };
    let help = Paragraph::new(help_text.as_ref()).style(if app.confirm_remove {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    });
    frame.render_widget(help, chunks[3]);

    if app.confirm_remove {
        let n = app.count_selected();
        let area = centered_rect(70, 40, frame.area());
        let clear = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(clear, area);

        let inner = area.inner(Margin::new(2, 1));
        let keys_line = Line::from(vec![
            Span::styled(
                " Y ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" confirm removal   "),
            Span::styled(
                " N ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" or "),
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" cancel"),
        ]);
        let body = Text::from(vec![
            keys_line,
            Line::raw(""),
            Line::raw(format!(
                "Remove {n} selected app(s) for the primary user (user 0)?"
            )),
            Line::raw(""),
            Line::raw(
                "This uses pm uninstall --user 0 (packages remain on the system image for this user).",
            ),
            Line::raw(
                "Restore may be possible via Play Store or adb install-existing where applicable.",
            ),
        ]);
        let block = Paragraph::new(body)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Confirm removal — Y / N / Esc ")
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(block, inner);
    }
}

fn truncate_desc(s: &str, max: usize) -> String {
    let mut t = s.to_string();
    if t.chars().count() > max {
        t = t.chars().take(max.saturating_sub(1)).collect::<String>() + "…";
    }
    t
}

/// `percent_x`, `percent_y` of full area
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Returns `true` if the UI should exit.
pub fn handle_key(app: &mut App, code: KeyCode) -> Result<bool> {
    if app.confirm_remove {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.confirm_remove = false;
                run_removal(app)?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.confirm_remove = false;
                app.status = "Cancelled.".into();
            }
            _ => {}
        }
        return Ok(false);
    }

    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
        KeyCode::Char('r') | KeyCode::Char('R') => refresh_packages(app, true)?,
        KeyCode::Char('A') => {
            app.show_advanced = !app.show_advanced;
            app.rebuild_visible();
            app.status = format!(
                "Advanced packages {}",
                if app.show_advanced { "visible" } else { "hidden" }
            );
        }
        KeyCode::Char('a') => {
            for &idx in &app.visible_indices {
                app.items[idx].selected = true;
            }
            app.status = "Selected all visible packages.".into();
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            for &idx in &app.visible_indices {
                app.items[idx].selected = false;
            }
            app.status = "Cleared selection (visible).".into();
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            let n = app.count_selected();
            if n == 0 {
                app.status = "No packages selected.".into();
            } else {
                app.confirm_remove = true;
                app.status = format!("Confirm removal of {n} package(s).");
            }
        }
        KeyCode::Up => {
            let n = app.visible_indices.len();
            if n == 0 {
                return Ok(false);
            }
            let i = app.list_state.selected().unwrap_or(0);
            let ni = if i == 0 { n - 1 } else { i - 1 };
            app.list_state.select(Some(ni));
        }
        KeyCode::Down => {
            let n = app.visible_indices.len();
            if n == 0 {
                return Ok(false);
            }
            let oi = app.list_state.selected().unwrap_or(0);
            let ni = if oi + 1 >= n { 0 } else { oi + 1 };
            app.list_state.select(Some(ni));
        }
        KeyCode::Char(' ') => {
            if let Some(si) = app.list_state.selected() {
                if let Some(&idx) = app.visible_indices.get(si) {
                    app.items[idx].selected = !app.items[idx].selected;
                }
            }
        }
        _ => {}
    }
    Ok(false)
}

fn refresh_packages(app: &mut App, clear_log: bool) -> Result<()> {
    if clear_log {
        app.log.clear();
    }
    let installed = adb::list_installed_packages(app.device_serial.as_deref())?;
    for b in &mut app.items {
        b.selected = false;
    }
    app.items.retain(|b| installed.contains(&b.meta.id));
    app.items.sort();
    app.rebuild_visible();
    app.status = format!("Refreshed — {} known bloat packages installed.", app.items.len());
    Ok(())
}

fn run_removal(app: &mut App) -> Result<()> {
    let pkgs: Vec<String> = app.selected_visible_pkgs();
    app.log.clear();
    let mut ok = 0usize;
    let mut fail = 0usize;
    for pkg in &pkgs {
        match adb::uninstall_user_zero(app.device_serial.as_deref(), pkg) {
            Ok(msg) => {
                app.log.push(format!("OK  {pkg}: {msg}"));
                ok += 1;
            }
            Err(e) => {
                app.log.push(format!("ERR {pkg}: {e}"));
                fail += 1;
            }
        }
    }
    for &idx in &app.visible_indices {
        app.items[idx].selected = false;
    }
    app.status = format!("Done. Success: {ok}, failed: {fail}. Press r to refresh list.");
    refresh_packages(app, false)?;
    Ok(())
}
