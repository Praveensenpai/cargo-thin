use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::domain::cleaner::CleanReport;
use crate::domain::project::format_bytes;
use crate::ui::app::App;
use crate::ui::theme::Theme;

pub fn render_confirm_modal(frame: &mut Frame, app: &App) {
    let theme = Theme::default();
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    let count = app.selected_count();
    let bytes = format_bytes(app.selected_reclaimable_bytes());

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠️  Confirm Safe Target Purge",
            theme.accent_amber,
        )),
        Line::from(""),
        Line::from(format!("Selected projects to purge: {count}")),
        Line::from(vec![
            Span::raw("Reclaimable disk space: "),
            Span::styled(bytes, theme.accent_green),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Compiled executable binaries will be staged and safely preserved.",
            theme.accent_cyan,
        )),
        Line::from(Span::styled(
            "Intermediate build caches (deps, incremental, build) will be removed.",
            theme.text_muted,
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [Enter / y] Confirm Purge ", theme.accent_green),
            Span::styled("   [Esc / n] Cancel ", theme.accent_red),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused)
        .title(" Confirmation ")
        .title_alignment(Alignment::Center);

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}

pub fn render_summary_modal(frame: &mut Frame, reports: &[CleanReport]) {
    let theme = Theme::default();
    let area = centered_rect(65, 45, frame.area());
    frame.render_widget(Clear, area);

    let total_freed: u64 = reports.iter().map(|r| r.bytes_freed).sum();
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("🎉 Cleanup Complete!", theme.accent_green)),
        Line::from(""),
        Line::from(vec![
            Span::raw("Total Space Reclaimed: "),
            Span::styled(format_bytes(total_freed), theme.accent_green),
        ]),
        Line::from(format!("Projects Cleaned: {}", reports.len())),
        Line::from(""),
    ];

    for r in reports.iter().take(5) {
        lines.push(Line::from(vec![
            Span::styled(format!("  ✔ {} ", r.package_name), theme.text_primary),
            Span::styled(
                format!(
                    "(-{}, {} bins kept)",
                    format_bytes(r.bytes_freed),
                    r.binaries_preserved
                ),
                theme.text_muted,
            ),
        ]));
    }

    if reports.len() > 5 {
        lines.push(Line::from(Span::styled(
            format!("  ...and {} more", reports.len() - 5),
            theme.text_muted,
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press [Enter] or [Esc] to continue",
        theme.accent_cyan,
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.accent_green)
        .title(" Summary ")
        .title_alignment(Alignment::Center);

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}

pub fn render_help_modal(frame: &mut Frame) {
    let theme = Theme::default();
    let area = centered_rect(55, 45, frame.area());
    frame.render_widget(Clear, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("Keyboard Navigation", theme.header)),
        Line::from(""),
        Line::from("  j / Down       Move cursor down"),
        Line::from("  k / Up         Move cursor up"),
        Line::from("  Space          Toggle project selection"),
        Line::from("  a              Select all / Deselect all"),
        Line::from("  s              Cycle sort order (Size / Savings / Name)"),
        Line::from("  p              Toggle policy (Prefer Release / Keep Both)"),
        Line::from("  c              Clean selected projects"),
        Line::from("  ?              Toggle this help dialog"),
        Line::from("  q / Ctrl+C     Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Press [Esc] or [?] to close",
            theme.accent_cyan,
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused)
        .title(" Help ")
        .title_alignment(Alignment::Center);

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn render_cleaning_modal(
    frame: &mut Frame,
    current: usize,
    total: usize,
    current_name: &str,
    tick: u8,
) {
    let theme = Theme::default();
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    const SPINNER: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let spinner = SPINNER[(tick as usize) % SPINNER.len()];
    let bar = build_progress_bar(current, total);

    let project_label = if current_name.is_empty() {
        "Starting…".to_string()
    } else {
        format!("Purging: {current_name}")
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {spinner}  Cleaning in Progress…"),
            theme.accent_cyan,
        )),
        Line::from(""),
        Line::from(format!("  Project {current} of {total}")),
        Line::from(Span::styled(
            format!("  {project_label}"),
            theme.text_primary,
        )),
        Line::from(""),
        Line::from(Span::styled(format!("  {bar}"), theme.accent_green)),
        Line::from(""),
        Line::from(Span::styled(
            "  Binaries are safely staged. Do not close the terminal.",
            theme.text_muted,
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.accent_cyan)
        .title(" Purging Targets ")
        .title_alignment(Alignment::Center);

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn build_progress_bar(current: usize, total: usize) -> String {
    let pct = if total > 0 {
        ((current as f64 / total as f64) * 100.0) as u16
    } else {
        0
    };
    let width = 36usize;
    let filled = (width * pct as usize / 100).min(width);
    format!(
        "[{}{}] {pct}%",
        "█".repeat(filled),
        "░".repeat(width - filled)
    )
}

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
