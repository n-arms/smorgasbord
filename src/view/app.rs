use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::App;
use crate::view::table::Table;

impl App {
    pub fn render(&self, f: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(f.size());

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let title = Paragraph::new(Text::styled(
            "Smorgasbord",
            Style::default().fg(Color::Green),
        ))
        .block(title_block);

        f.render_widget(title, chunks[0]);

        let table = Table::new(
            self.grid.get_widgets(),
            self.grid.get_width(),
            self.grid.get_height(),
            self.cursor,
        );

        f.render_widget(table, chunks[1]);

        let backend_log = Paragraph::new(format!("{:?}", self.network_table))
            .block(Block::new().borders(Borders::ALL));

        f.render_widget(backend_log, chunks[2]);
    }
}
