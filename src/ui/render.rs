use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, TableState};
use ratatui::Frame;

use crate::domain::project::{format_bytes, ProjectInfo};
use crate::ui::app::{App, AppMode};
use crate::ui::modals::{
    render_cleaning_modal, render_confirm_modal, render_help_modal, render_summary_modal,
};
use crate::ui::theme::{standard_block, Theme};

pub fn render_app(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_body(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);

    match &app.mode {
        AppMode::ConfirmModal => render_confirm_modal(frame, app),
        AppMode::SummaryModal(reports) => render_summary_modal(frame, reports),
        AppMode::HelpModal => render_help_modal(frame),
        AppMode::Cleaning {
            current,
            total,
            current_name,
            tick,
            ..
        } => render_cleaning_modal(frame, *current, *total, current_name, *tick),
        AppMode::Browsing => {}
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();
    let total_target = format_bytes(app.total_target_bytes());
    let total_reclaim = format_bytes(app.total_reclaimable_bytes());
    let percent = if app.total_target_bytes() > 0 {
        (app.total_reclaimable_bytes() as f64 / app.total_target_bytes() as f64) * 100.0
    } else {
        0.0
    };

    let profile_mode = if app.keep_both {
        "Keep Both"
    } else {
        "Prefer Release"
    };

    let title_line = Line::from(vec![
        Span::styled(" Cargo Thin (カーゴ・シン) ", theme.title),
        Span::styled("│ Projects: ", theme.text_muted),
        Span::styled(format!("{}", app.projects.len()), theme.accent_cyan),
        Span::styled(" │ Target Bloat: ", theme.text_muted),
        Span::styled(total_target, theme.accent_amber),
        Span::styled(" │ Reclaimable: ", theme.text_muted),
        Span::styled(
            format!("{total_reclaim} ({percent:.0}%)"),
            theme.accent_green,
        ),
        Span::styled(format!(" │ Policy: [{profile_mode}]"), theme.text_muted),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .padding(ratatui::widgets::Padding::horizontal(1));

    frame.render_widget(Paragraph::new(title_line).block(block), area);
}

fn render_body(frame: &mut Frame, app: &App, area: Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_project_table(frame, app, body_chunks[0]);
    render_inspector(frame, app, body_chunks[1]);
}

fn render_project_table(frame: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();
    let header_cells = ["", "Project", "Target", "Bins", "Savings"]
        .into_iter()
        .map(|h| Span::styled(h, theme.header));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .projects
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let checkbox = if p.selected { "[✓] " } else { "[ ] " };
            let cb_style = if p.selected {
                theme.accent_cyan
            } else {
                theme.text_muted
            };

            let name_style = if idx == app.selected_index {
                theme.selected_item
            } else {
                theme.text_primary
            };

            let percent = p.reclaimable_percent(app.keep_both);
            let gauge_str = format!("{percent:>3.0}%");

            Row::new(vec![
                Span::styled(checkbox, cb_style),
                Span::styled(&p.package_name, name_style),
                Span::styled(format_bytes(p.total_target_bytes), theme.accent_amber),
                Span::styled(format!("{}", p.binaries.len()), theme.text_muted),
                Span::styled(gauge_str, theme.accent_green),
            ])
            .height(1)
        })
        .collect();

    let table_title = format!(" Projects (Sort: {}) ", app.sort_order.label());
    let block = standard_block(&table_title, true);

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Min(16),
            Constraint::Length(12),
            Constraint::Length(6),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(block)
    .highlight_symbol(theme.highlight_symbol);

    let mut state = TableState::default();
    state.select(Some(app.selected_index));
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_inspector(frame: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();
    let Some(project) = app.selected_project() else {
        let block = standard_block(" Project Inspector ", false);
        frame.render_widget(Paragraph::new("No project selected").block(block), area);
        return;
    };

    let title = format!(" Inspector: {} ", project.package_name);
    let block = standard_block(&title, false);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Location: ", theme.text_muted),
            Span::styled(project.root_path.to_string_lossy(), theme.text_primary),
        ]),
        Line::from(""),
    ];

    append_inspector_breakdown(project, &theme, &mut lines);
    append_inspector_binaries(project, &theme, &mut lines);

    let reclaimable = format_bytes(project.reclaimable_bytes(app.keep_both));
    let pct = project.reclaimable_percent(app.keep_both);
    lines.push(Line::from(vec![
        Span::styled("Reclaimable: ", theme.header),
        Span::styled(format!("{reclaimable} ({pct:.1}%)"), theme.accent_green),
    ]));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn append_inspector_breakdown<'a>(project: &ProjectInfo, theme: &Theme, lines: &mut Vec<Line<'a>>) {
    lines.push(Line::from(Span::styled(
        "── Target Bloat Breakdown ──",
        theme.header,
    )));
    lines.push(Line::from(vec![
        Span::styled("  deps/          ", theme.text_muted),
        Span::styled(
            format_bytes(project.breakdown.deps_bytes),
            theme.accent_amber,
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  incremental/   ", theme.text_muted),
        Span::styled(
            format_bytes(project.breakdown.incremental_bytes),
            theme.accent_amber,
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  build/         ", theme.text_muted),
        Span::styled(
            format_bytes(project.breakdown.build_bytes),
            theme.accent_amber,
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  .fingerprint/  ", theme.text_muted),
        Span::styled(
            format_bytes(project.breakdown.fingerprint_bytes),
            theme.accent_amber,
        ),
    ]));
    lines.push(Line::from(""));
}

fn append_inspector_binaries<'a>(project: &ProjectInfo, theme: &Theme, lines: &mut Vec<Line<'a>>) {
    lines.push(Line::from(Span::styled(
        "── Preserved Binaries (ELF) ──",
        theme.header,
    )));
    if project.binaries.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (No compiled binaries found)",
            theme.text_muted,
        )));
    } else {
        for b in &project.binaries {
            lines.push(Line::from(vec![
                Span::styled("  ✓ [ELF] ", theme.accent_green),
                Span::styled(format!("{}/{} ", b.profile, b.filename), theme.text_primary),
                Span::styled(
                    format!("({})", format_bytes(b.size_bytes)),
                    theme.text_muted,
                ),
            ]));
        }
    }
    lines.push(Line::from(""));
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();
    let status_text = if let Some(msg) = &app.status_message {
        Line::from(Span::styled(msg, theme.accent_amber))
    } else {
        Line::from(vec![
            Span::styled("[Space] ", theme.accent_cyan),
            Span::styled("Toggle  ", theme.text_primary),
            Span::styled("[a] ", theme.accent_cyan),
            Span::styled("All  ", theme.text_primary),
            Span::styled("[s] ", theme.accent_cyan),
            Span::styled("Sort  ", theme.text_primary),
            Span::styled("[p] ", theme.accent_cyan),
            Span::styled("Policy  ", theme.text_primary),
            Span::styled("[c] ", theme.accent_green),
            Span::styled("Clean Selected  ", theme.text_primary),
            Span::styled("[?] ", theme.accent_cyan),
            Span::styled("Help  ", theme.text_primary),
            Span::styled("[q] ", theme.accent_red),
            Span::styled("Quit", theme.text_primary),
        ])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .padding(ratatui::widgets::Padding::horizontal(1));

    frame.render_widget(Paragraph::new(status_text).block(block), area);
}
