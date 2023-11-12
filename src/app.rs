use anyhow::Result;
use crossterm::event::{self, Event::Key, KeyCode::Char};
use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

use crate::{grid::ManagedGrid, nt_backend::Backend, table::Table, widget_manager::make_widgets};

pub struct App {
    grid: ManagedGrid,
    network_table: Backend,
}

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
        );

        f.render_widget(table, chunks[1]);

        let backend_log = Paragraph::new(format!("{:?}", self.network_table))
            .block(Block::new().borders(Borders::ALL));

        f.render_widget(backend_log, chunks[2]);
    }

    pub async fn update(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(250))? {
            if let Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    return match key.code {
                        Char('q') => Ok(true),
                        _ => Ok(false),
                    };
                }
            }
        }
        let widgets = self.network_table.with_keys(make_widgets);
        self.grid.populate_from(widgets);
        Ok(false)
    }

    pub fn new(network_table: Backend) -> App {
        Self {
            grid: ManagedGrid::new(5, 2),
            network_table,
        }
    }
}
