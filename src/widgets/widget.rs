use anyhow::Error;
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, StatefulWidget, Widget as UIWidget},
};
use std::fmt::Debug;

use crate::{
    nt::{Key, Path, Write},
    widget_tree::{Node, Value},
};

#[derive(Debug)]
pub struct Widget {
    pub title: Path,
    pub value: Box<dyn Kind>,
}

#[derive(Copy, Clone, Debug)]
pub enum State {
    Unhighlighted,
    Highlighted,
    Selected,
}

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

pub trait Kind: Debug {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn prompt(&self) -> String;
    fn update(&mut self, path: &Path, text: &str) -> Write;
    fn update_nt(&mut self, key: &Key, value: &Value);
    fn reset(&mut self);
    fn is_finished(&self) -> bool;
    fn size(&self) -> Size;
    fn clone_box(&self) -> Box<dyn Kind>;
}

pub enum BuildResult {
    Complete(Box<dyn Kind>),
    Partial(Error),
    None,
}

pub trait Builder {
    fn create_kind(&self, key: &Key, value: &Value) -> BuildResult;
}

const UNHIGHLIGHTED_COLOR: Color = Color::White;
const HIGHLIGHTED_COLOR: Color = Color::Yellow;
const SELECTED_COLOR: Color = Color::Magenta;

impl StatefulWidget for Widget {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut State) {
        let color = match state {
            State::Unhighlighted => UNHIGHLIGHTED_COLOR,
            State::Highlighted => HIGHLIGHTED_COLOR,
            State::Selected => SELECTED_COLOR,
        };

        let style = Style::default().fg(color);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title(self.title.to_string())
            .title_style(style);
        self.value.render(block.inner(area), buf);
        block.render(area, buf);
    }
}

impl Clone for Widget {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            value: self.value.clone_box(),
        }
    }
}

impl Widget {
    pub fn new(title: Path, value: Box<dyn Kind>) -> Self {
        Self { title, value }
    }

    pub fn reset(&mut self) {
        self.value.reset();
    }

    pub fn prompt(&self) -> String {
        self.value.prompt()
    }

    pub fn update(&mut self, text: &str) -> Write {
        self.value.update(&self.title, text)
    }

    pub fn update_nt(&mut self, key: &Key, value: &Value) {
        self.value.update_nt(key, value)
    }

    pub fn is_finished(&self) -> bool {
        self.value.is_finished()
    }

    pub fn size(&self) -> Size {
        self.value.size()
    }
}
