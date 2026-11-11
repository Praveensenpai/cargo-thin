use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Padding};

pub struct Theme {
    pub border: Style,
    pub border_focused: Style,
    pub title: Style,
    pub header: Style,
    pub text_primary: Style,
    pub text_muted: Style,
    pub selected_item: Style,
    pub highlight_symbol: &'static str,
    pub accent_cyan: Style,
    pub accent_green: Style,
    pub accent_amber: Style,
    pub accent_red: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: Style::default().fg(Color::Rgb(80, 80, 95)),
            border_focused: Style::default()
                .fg(Color::Rgb(100, 200, 255))
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(Color::Rgb(240, 240, 255))
                .add_modifier(Modifier::BOLD),
            header: Style::default()
                .fg(Color::Rgb(170, 220, 255))
                .add_modifier(Modifier::BOLD),
            text_primary: Style::default().fg(Color::Rgb(230, 230, 235)),
            text_muted: Style::default().fg(Color::Rgb(130, 135, 150)),
            selected_item: Style::default()
                .bg(Color::Rgb(40, 50, 75))
                .fg(Color::Rgb(255, 255, 255))
                .add_modifier(Modifier::BOLD),
            highlight_symbol: "▶ ",
            accent_cyan: Style::default().fg(Color::Rgb(80, 210, 255)),
            accent_green: Style::default()
                .fg(Color::Rgb(90, 230, 140))
                .add_modifier(Modifier::BOLD),
            accent_amber: Style::default()
                .fg(Color::Rgb(255, 195, 75))
                .add_modifier(Modifier::BOLD),
            accent_red: Style::default()
                .fg(Color::Rgb(255, 100, 110))
                .add_modifier(Modifier::BOLD),
        }
    }
}

pub fn standard_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let theme = Theme::default();
    let border_style = if focused {
        theme.border_focused
    } else {
        theme.border
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(theme.title)
        .padding(Padding::horizontal(1))
}
