use network_tables::Value;
use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{
    nt::Key,
    trie::{Node, NodeValue},
};

use super::widget::{self, Kind};

#[derive(Clone, Debug)]
pub struct Simple {
    value: Value,
}

impl Kind for Simple {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let widget = Paragraph::new(self.value.to_string());
        widget.render(area, buf);
    }

    fn prompt(&self) -> String {
        String::from("Simple widgets are constant")
    }

    fn update(&mut self, _text: &str) {}

    fn update_nt(&mut self, nt: &Node<Key, Value>) {
        if let NodeValue::Leaf(value) = &nt.value {
            self.value = value.clone();
        }
    }

    fn reset(&mut self) {}

    fn is_finished(&self) -> bool {
        true
    }

    fn clone_box(&self) -> Box<dyn Kind> {
        Box::new(self.clone())
    }
}

pub struct Builder;

impl widget::Builder for Builder {
    fn create_kind(&self, nt: &Node<Key, Value>) -> Option<Box<dyn Kind>> {
        let NodeValue::Leaf(value) = &nt.value else {
            return None
        };

        if nt.key.starts_with(".") {
            None
        } else {
            Some(Box::new(Simple {
                value: value.clone(),
            }))
        }
    }
}
