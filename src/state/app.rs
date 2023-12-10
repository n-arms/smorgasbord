use anyhow::Result;
use crossterm::event::KeyCode;
use crossterm::event::{self, Event::Key, KeyCode::Char};
use std::time::Duration;

use crate::nt::{Backend, Update};
use crate::state::{
    grid::{GridPosition, ManagedGrid},
    widget_manager::WidgetManager,
};
use crate::widget_tree::Tree;
use crate::widgets::{self, sendable_chooser, simple, Size};

use thiserror::Error;
use tui_input::{Input, InputRequest};

use super::pack::Packer;
use super::packing::Packing;

pub struct App {
    pub packing: Packing,
    pub network_table: Backend,
    pub widget_tree: Tree,
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
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cursor {cursor:?} is invalid")]
    InvalidCursor { cursor: GridPosition },
    #[error("Widget at position {position:?} disappeared while editting")]
    RunawayWidget { position: GridPosition },
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
                                self.cursor.x =
                                    (self.cursor.x + 1).min(self.packing.get_width() - 1);
                            }
                            KeyCode::Up => {
                                self.cursor.y = self.cursor.y.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                self.cursor.y =
                                    (self.cursor.y + 1).min(self.packing.get_height() - 1);
                            }
                            KeyCode::Enter => {
                                self.try_edit();
                            }
                            _ => {}
                        },
                        State::Edit(edit) => {
                            let Some(widget) = self.packing.get_mut_widget(edit.editting) else {
                                return Err(Error::RunawayWidget {
                                    position: edit.editting,
                                }
                                .into());
                            };
                            match key.code {
                                KeyCode::Enter => {
                                    self.network_table
                                        .write(widget.update(edit.text_field.value()));
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
                                    self.state = State::View;
                                    return Ok(false);
                                }
                                _ => {}
                            };
                            if widget.is_finished() {
                                self.state = State::View;
                            }
                        }
                    }
                }
            }
        }
        let Update {
            to_update,
            to_create,
        } = self.network_table.update();

        for entry in to_update.into_iter().chain(to_create) {
            self.widget_tree.update_entry(entry)?;
        }

        let all_widgets = self.widget_tree.widgets();

        for widget in all_widgets {
            self.packing.add(widget.clone());
        }

        Ok(false)
    }

    fn try_edit(&mut self) {
        if let Some(widget) = self.packing.get_mut_widget(self.cursor) {
            widget.reset();
            self.state = State::Edit(Edit {
                editting: self.cursor,
                text_field: Input::new(String::new()),
                prompt: widget.prompt(),
            });
        }
    }

    fn check_health(&mut self) -> Result<()> {
        if self.cursor.x >= self.packing.get_width() {
            self.cursor = GridPosition::default();
            return Err(Error::InvalidCursor {
                cursor: self.cursor,
            }
            .into());
        }
        if self.cursor.y >= self.packing.get_height() {
            self.cursor = GridPosition::default();
            return Err(Error::InvalidCursor {
                cursor: self.cursor,
            }
            .into());
        }
        Ok(())
    }

    pub fn new(network_table: Backend) -> App {
        let builders: Vec<Box<dyn widgets::Builder>> = vec![
            Box::new(simple::Builder),
            Box::new(sendable_chooser::Builder),
        ];
        let widget_tree = Tree::new(builders);
        Self {
            packing: Packing::new(Size {
                width: 4,
                height: 2,
            }),
            network_table,
            cursor: GridPosition::default(),
            state: State::View,
            widget_tree,
        }
    }
}
