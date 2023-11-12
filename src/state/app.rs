use anyhow::Result;
use crossterm::event::{self, Event::Key, KeyCode::Char};
use std::time::Duration;

use crate::nt_backend::Backend;
use crate::state::{
    grid::{GridPosition, ManagedGrid},
    widget_manager::make_widgets,
};

use thiserror::Error;

pub struct App {
    pub grid: ManagedGrid,
    pub network_table: Backend,
    pub cursor: GridPosition,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cursor {cursor:?} is invalid")]
    InvalidCursor { cursor: GridPosition },
}

impl App {
    pub async fn update(&mut self) -> Result<bool> {
        self.check_health()?;
        if event::poll(Duration::from_millis(250))? {
            if let Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        Char('q') => return Ok(true),
                        event::KeyCode::Left => {
                            self.cursor.x = self.cursor.x.saturating_sub(1);
                        }
                        event::KeyCode::Right => {
                            self.cursor.x = (self.cursor.x + 1).min(self.grid.get_width() - 1);
                        }
                        event::KeyCode::Up => {
                            self.cursor.y = self.cursor.y.saturating_sub(1);
                        }
                        event::KeyCode::Down => {
                            self.cursor.y = (self.cursor.y + 1).min(self.grid.get_height() - 1);
                        }
                        _ => {}
                    };
                }
            }
        }
        let widgets = self.network_table.with_keys(make_widgets);
        self.grid.populate_from(widgets);
        Ok(false)
    }

    fn check_health(&mut self) -> Result<()> {
        if self.cursor.x >= self.grid.get_width() {
            self.cursor = GridPosition::default();
            return Err(Error::InvalidCursor {
                cursor: self.cursor,
            }
            .into());
        }
        if self.cursor.y >= self.grid.get_height() {
            self.cursor = GridPosition::default();
            return Err(Error::InvalidCursor {
                cursor: self.cursor,
            }
            .into());
        }
        Ok(())
    }

    pub fn new(network_table: Backend) -> App {
        Self {
            grid: ManagedGrid::new(5, 2),
            network_table,
            cursor: GridPosition::default(),
        }
    }
}
