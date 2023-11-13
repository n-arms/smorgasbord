use network_tables::Value;
use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{nt::Key, trie::NodeValue};

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

    fn update_nt(&mut self, nt: &NodeValue<Key, Value>) {
        if let NodeValue::Leaf(value) = nt {
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
    fn create_kind(&self, nt: &NodeValue<Key, Value>) -> Option<Box<dyn Kind>> {
        let NodeValue::Leaf(value) = nt else {
            return None
        };

        Some(Box::new(Simple {
            value: value.clone(),
        }))
    }
}
