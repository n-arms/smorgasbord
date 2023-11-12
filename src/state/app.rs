use anyhow::Result;
use crossterm::event::KeyCode;
use crossterm::event::{self, Event::Key, KeyCode::Char};
use std::time::Duration;

use crate::nt_backend::Backend;
use crate::state::{
    grid::{GridPosition, ManagedGrid},
    widget_manager::make_widgets,
};

use thiserror::Error;
use tui_input::{Input, InputRequest};

use crate::view::widget::WidgetState;

pub struct App {
    pub grid: ManagedGrid,
    pub network_table: Backend,
    pub cursor: GridPosition,
    pub state: State,
}

pub enum State {
    View,
    Edit(Edit),
}

pub struct Edit {
    pub editting: GridPosition,
    pub text_field: Input,
    pub prompt: String,
    pub state: WidgetState,
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
                    match &mut self.state {
                        State::View => match key.code {
                            Char('q') => return Ok(true),
                            KeyCode::Left => {
                                self.cursor.x = self.cursor.x.saturating_sub(1);
                            }
                            KeyCode::Right => {
                                self.cursor.x = (self.cursor.x + 1).min(self.grid.get_width() - 1);
                            }
                            KeyCode::Up => {
                                self.cursor.y = self.cursor.y.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                self.cursor.y = (self.cursor.y + 1).min(self.grid.get_height() - 1);
                            }
                            KeyCode::Enter => {
                                self.try_edit();
                            }
                            _ => {}
                        },
                        State::Edit(edit) => {
                            match key.code {
                                KeyCode::Enter => {
                                    edit.state.with_text(edit.text_field.value());
                                    edit.text_field.reset();
                                }
                                KeyCode::Left => {
                                    edit.text_field.handle(InputRequest::GoToPrevChar);
                                }
                                KeyCode::Right => {
                                    edit.text_field.handle(InputRequest::GoToNextChar);
                                }
                                KeyCode::Backspace => {
                                    edit.text_field.handle(InputRequest::DeletePrevChar);
                                }
                                Char(c) => {
                                    edit.text_field.handle(InputRequest::InsertChar(c));
                                }
                                KeyCode::Esc => {
                                    edit.state.is_finished = true;
                                }
                                _ => {}
                            };
                            if edit.state.is_finished {
                                self.state = State::View;
                            }
                        }
                    }
                }
            }
        }
        let widgets = self.network_table.with_keys(make_widgets);
        self.grid.populate_from(widgets);
        Ok(false)
    }

    fn try_edit(&mut self) {
        if self.grid.get_widget(&self.cursor).is_some() {
            let widget_state = WidgetState::editting();
            self.state = State::Edit(Edit {
                editting: self.cursor,
                text_field: Input::new(String::new()),
                prompt: widget_state.prompt(),
                state: widget_state,
            });
        }
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
            state: State::View,
        }
    }
}