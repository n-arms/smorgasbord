use network_tables::Value;
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style},
    widgets::{List, ListItem, ListState, StatefulWidget},
};

use crate::{
    nt::{Entry, Key, Path, Write},
    trie::Node,
};

use super::{
    util::NTValue,
    widget::{self, Kind},
};

#[derive(Clone, Debug)]
pub struct SendableChooser {
    options: Vec<String>,
    active: Option<usize>,
    default: usize,
    is_finished: bool,
}

#[derive(Debug)]
pub enum Error {
    WantedNodes,
    MissingType,
    TypeNotString,
    IncorrectType(String),
    MissingOptions,
    MissingDefault,
    IllegalSelection,
}

impl TryFrom<&Node<Key, Value>> for SendableChooser {
    type Error = Error;

    fn try_from(data: &Node<Key, Value>) -> Result<Self, Self::Error> {
        let nodes = data.value.try_get_nodes().ok_or(Error::WantedNodes)?;
        let r#type = nodes
            .try_get_value(".type")
            .ok_or(Error::MissingType)?
            .try_to_string()
            .ok_or(Error::TypeNotString)?;

        if r#type != "String Chooser" {
            return Err(Error::IncorrectType(r#type));
        }

        let options = nodes
            .try_get_value("options")
            .and_then(|value| value.try_to_string_array())
            .ok_or(Error::MissingOptions)?;
        let active_name = nodes
            .try_get_value("selected")
            .and_then(|value| value.try_to_string());
        let default_name = nodes
            .try_get_value("default")
            .and_then(|value| value.try_to_string())
            .ok_or(Error::MissingDefault)?;

        let active = options.iter().enumerate().find_map(|(i, value)| {
            if Some(value) == active_name.as_ref() {
                Some(i)
            } else {
                None
            }
        });

        let default = options
            .iter()
            .enumerate()
            .find_map(|(i, value)| {
                if value == &default_name {
                    Some(i)
                } else {
                    None
                }
            })
            .ok_or(Error::IllegalSelection)?;

        Ok(SendableChooser {
            options,
            active,
            default,
            is_finished: false,
        })
    }
}

impl Kind for SendableChooser {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let items: Vec<_> = self
            .options
            .iter()
            .map(String::as_str)
            .map(ListItem::new)
            .collect();
        let index = self.active.unwrap_or(self.default);

        let widget = List::new(items)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::LightBlue));

        let mut state = ListState::default().with_selected(Some(index));

        StatefulWidget::render(widget, area, buf, &mut state);
    }

    fn prompt(&self) -> String {
        String::from("Enter an option")
    }

    fn update(&mut self, path: &Path, text: &str) -> Write {
        for (i, option) in self.options.iter().enumerate() {
            if option == text {
                self.active = Some(i);
                self.is_finished = true;
                let mut path = path.clone();
                path.push("selected");
                return Write::one(Entry {
                    path,
                    value: Value::String(text.into()),
                });
            }
        }
        Write::default()
    }

    fn update_nt(&mut self, nt: &Node<Key, Value>) {
        let Ok(value) = SendableChooser::try_from(nt) else {
            todo!();
        };
        self.options = value.options;
        self.default = value.default;
    }

    fn reset(&mut self) {
        self.is_finished = false;
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn clone_box(&self) -> Box<dyn Kind> {
        Box::new(self.clone())
    }
}

pub struct Builder;

impl widget::Builder for Builder {
    fn create_kind(&self, nt: &Node<Key, Value>) -> Option<Box<dyn Kind>> {
        let widget = SendableChooser::try_from(nt).ok()?;
        Some(Box::new(widget))
    }
}
