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

use crate::{grid::ManagedGrid, nt_backend::Backend};

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

        let mut grid_constraints = Vec::new();
        let mut grid_widgets = Vec::new();

        for (_, widget) in self.grid.get_widgets() {
            grid_constraints.push(Constraint::Length(16));

            let block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());

            let text = match widget.value {
                Some(value) => value.to_string(),
                None => "".into(),
            };

            let tui_widget =
                Paragraph::new(Text::styled(text, Style::default().fg(Color::Yellow))).block(block);

            grid_widgets.push(tui_widget);
        }

        let grid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(grid_constraints)
            .split(chunks[1]);

        for (i, widget) in grid_widgets.into_iter().enumerate() {
            f.render_widget(widget, grid[i]);
        }

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
        self.grid.populate_from(self.network_table.widgets());
        Ok(false)
    }

    pub fn new(network_table: Backend) -> App {
        Self {
            grid: ManagedGrid::new(10, 1),
            network_table,
        }
    }
}
