use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ui::app::{App, AppMode};

pub fn handle_event(app: &mut App, key: KeyEvent) {
    match &app.mode {
        AppMode::Browsing => handle_browsing_keys(app, key),
        AppMode::ConfirmModal => handle_confirm_keys(app, key),
        AppMode::Cleaning { .. } => {} // ignore all keys while cleaning
        AppMode::SummaryModal(_) => handle_summary_keys(app, key),
        AppMode::HelpModal => handle_help_keys(app, key),
    }
}

fn handle_browsing_keys(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.move_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char(' ') => app.toggle_selection(),
        KeyCode::Char('a') => app.toggle_all(),
        KeyCode::Char('s') => app.cycle_sort(),
        KeyCode::Char('p') => app.toggle_keep_both(),
        KeyCode::Char('c') => {
            if app.selected_count() > 0 {
                app.mode = AppMode::ConfirmModal;
            } else {
                app.status_message =
                    Some("No projects selected. Press [Space] or [a].".to_string());
            }
        }
        KeyCode::Char('?') => app.mode = AppMode::HelpModal,
        _ => {}
    }
}

fn handle_confirm_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter | KeyCode::Char('y') => {
            app.start_clean();
        }
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
            app.mode = AppMode::Browsing;
        }
        _ => {}
    }
}

fn handle_summary_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char(' ') => {
            app.mode = AppMode::Browsing;
        }
        _ => {}
    }
}

fn handle_help_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            app.mode = AppMode::Browsing;
        }
        _ => {}
    }
}
