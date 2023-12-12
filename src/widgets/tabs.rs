use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};
use thiserror::Error;

use crate::{
    backend::{Key, Path, PathError, Write},
    widget_tree::Value,
    widgets,
};

use super::{util::NTValue, widget, BuildResult, Kind, Size};

#[derive(Clone, Debug)]
struct Tab {
    option: String,
    widgets: Vec<Path>,
}

#[derive(Clone, Debug)]
pub struct Tabs {
    options: Vec<Tab>,
    selected: Option<usize>,
    is_finished: bool,
}

#[derive(Clone, Debug)]
pub enum Filter {
    NoneOf(Vec<Path>),
    OneOf(Vec<Path>),
}

impl Filter {
    pub fn contains(&self, widget: &widgets::Widget) -> bool {
        match self {
            Self::NoneOf(paths) => !paths.contains(&widget.title),
            Self::OneOf(paths) => paths.contains(&widget.title),
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::NoneOf(Vec::new())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("want nodes")]
    WantedNodes,
    #[error("missing .type")]
    MissingType,
    #[error(".type isn't a string")]
    TypeNotString,
    #[error(".type is {0}")]
    IncorrectType(String),
    #[error("want value")]
    WantedValue,
    #[error("option isn't a string array")]
    OptionNotStringArray,
    #[error("parsing error {0:?} while parsing a path")]
    PathParse(PathError),
}

impl Tabs {
    #[allow(clippy::option_if_let_else)]
    pub fn filter(&self) -> Filter {
        if let Some(index) = self.selected {
            let paths = &self.options[index].widgets;

            Filter::OneOf(paths.clone())
        } else {
            let mut paths = Vec::new();

            for tab in &self.options {
                paths.extend(tab.widgets.iter().cloned());
            }

            Filter::NoneOf(paths)
        }
    }
}

impl TryFrom<&Value> for Tabs {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let nodes = value.try_get_nodes().ok_or(Error::WantedNodes)?;
        let r#type = nodes
            .try_get_value(".type")
            .ok_or(Error::MissingType)?
            .try_to_string()
            .ok_or(Error::TypeNotString)?;

        if r#type != "Tabs" {
            return Err(Error::IncorrectType(r#type));
        }

        let mut options = Vec::new();

        for node in nodes.iter() {
            if node.key.as_str().starts_with('.') {
                continue;
            }
            let option = node.key.clone();
            let inner = node
                .value
                .try_get_value()
                .ok_or(Error::WantedValue)?
                .try_to_string_array()
                .ok_or(Error::OptionNotStringArray)?;

            let widgets: Result<Vec<_>, _> = inner.into_iter().map(|path| path.parse()).collect();

            options.push(Tab {
                option: option.to_string(),
                widgets: widgets.map_err(Error::PathParse)?,
            });
        }

        Ok(Self {
            options,
            selected: None,
            is_finished: false,
        })
    }
}

impl Kind for Tabs {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let constraints = vec![Constraint::Length(1); self.options.len() + 1];
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let style = if self.selected.is_none() {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default().fg(Color::White)
        };
        let width = Paragraph::new("General").style(style);
        width.render(chunks[0], buf);

        for (i, (chunk, option)) in chunks.iter().skip(1).zip(&self.options).enumerate() {
            let style = if Some(i) == self.selected {
                Style::default().fg(Color::LightBlue)
            } else {
                Style::default().fg(Color::White)
            };
            let widget = Paragraph::new(option.option.as_str()).style(style);
            widget.render(*chunk, buf);
        }
    }

    fn prompt(&self) -> String {
        String::from("Enter an option")
    }

    fn update(&mut self, path: &Path, text: &str) -> Write {
        let index = self.options.iter().enumerate().find_map(|(i, option)| {
            if option.option == text {
                Some(i)
            } else {
                None
            }
        });
        if let Some(index) = index {
            self.is_finished = true;
            self.selected = Some(index);

            let mut paths = self.options[index].widgets.clone();
            paths.push(path.clone());

            Write::filter(Filter::OneOf(paths))
        } else if text == "General" {
            self.is_finished = true;
            self.selected = None;

            Write::filter(self.filter())
        } else {
            Write::default()
        }
    }

    fn update_nt(&mut self, _key: &Key, value: &Value) {
        if let Ok(updated) = Self::try_from(value) {
            self.options = updated.options;
        }
    }

    fn reset(&mut self) {
        self.is_finished = false;
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn size(&self) -> Size {
        Size {
            width: 1,
            height: 2,
        }
    }

    fn clone_box(&self) -> Box<dyn Kind> {
        Box::new(self.clone())
    }
}

pub struct Builder;

impl widget::Builder for Builder {
    fn create_kind(&self, _key: &Key, value: &Value) -> BuildResult {
        let widget = Tabs::try_from(value);
        match widget {
            Ok(widget) => BuildResult::Complete(Box::new(widget)),
            Err(error @ (Error::OptionNotStringArray | Error::PathParse(_))) => {
                BuildResult::Partial(error.into())
            }
            Err(_) => BuildResult::None,
        }
    }
}
