mod adb;
mod model;
mod ui;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use model::{load_catalog, InstalledBloat, PackageEntry};
use ratatui::prelude::*;
use std::io::{stdout, Write};
use std::time::Duration;
use ui::App;

fn main() -> Result<()> {
    let catalog = load_catalog().context("load embedded package catalog")?;
    adb::adb_version().context("ADB not available")?;
    let serial = adb::resolve_device_serial()?;

    let model = adb::get_prop(serial.as_deref(), "ro.product.model").unwrap_or_default();
    let brand = adb::get_prop(serial.as_deref(), "ro.product.brand").unwrap_or_default();
    let android = adb::get_prop(serial.as_deref(), "ro.build.version.release").unwrap_or_default();
    let device_info = format!(
        "{} {} (Android {}) {}",
        brand,
        model,
        android,
        serial
            .as_ref()
            .map(|s| format!("serial {s}"))
            .unwrap_or_default()
    );

    let installed = adb::list_installed_packages(serial.as_deref())?;
    let mut items: Vec<InstalledBloat> = catalog
        .into_iter()
        .filter(|p: &PackageEntry| installed.contains(&p.id))
        .map(|meta| InstalledBloat {
            meta,
            selected: false,
        })
        .collect();
    items.sort();

    let mut app = App {
        items,
        visible_indices: Vec::new(),
        list_state: ratatui::widgets::ListState::default(),
        show_advanced: false,
        confirm_remove: false,
        device_serial: serial,
        device_info,
        status: String::new(),
        log: Vec::new(),
    };
    app.rebuild_visible();
    app.status = format!(
        "Ready — {} catalog packages currently installed. Toggle Advanced with A.",
        app.items.len()
    );

    enable_raw_mode().context("enable terminal raw mode")?;
    let mut term_out = stdout();
    execute!(term_out, EnterAlternateScreen).context("enter alternate screen")?;
    let backend = CrosstermBackend::new(term_out);
    let mut terminal = Terminal::new(backend).context("create terminal")?;

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    let mut out = stdout();
    execute!(out, LeaveAlternateScreen).ok();
    out.flush().ok();

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && ui::handle_key(app, key.code)? {
                break;
            }
        }
    }
    Ok(())
}
