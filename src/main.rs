use clap::Parser;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, stdout, IsTerminal};
use std::time::Duration;

use cargo_thin::cli::{run_headless, CliArgs};
use cargo_thin::domain::scan_projects;
use cargo_thin::ui::app::AppMode;
use cargo_thin::ui::{handle_event, render_app, App};

fn main() -> io::Result<()> {
    let mut raw_args: Vec<String> = std::env::args().collect();
    if raw_args.len() > 1 && raw_args[1] == "thin" {
        raw_args.remove(1);
    }
    let args = CliArgs::parse_from(raw_args);

    // Headless mode if flags set or stdout is piped
    if args.dry_run || args.json || args.plain || args.yes || !stdout().is_terminal() {
        return run_headless(&args);
    }

    let canonical = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());
    let projects = scan_projects(&canonical);

    // Install terminal recovery panic hook
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        default_hook(info);
    }));

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, crossterm::cursor::Hide)?;

    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(projects, args.keep_both);

    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;
    terminal.show_cursor()?;

    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while !app.should_quit {
        // Poll the background clean channel on every tick
        if matches!(app.mode, AppMode::Cleaning { .. }) {
            app.poll_clean();
        }

        terminal.draw(|f| render_app(f, app))?;

        // Faster tick while cleaning (80ms) to animate spinner smoothly;
        // normal browsing tick (100ms) is comfortable for keyboard nav
        let tick_ms = if matches!(app.mode, AppMode::Cleaning { .. }) {
            80
        } else {
            100
        };

        if event::poll(Duration::from_millis(tick_ms))? {
            if let Event::Key(key) = event::read()? {
                handle_event(app, key);
            }
        }
    }
    Ok(())
}
