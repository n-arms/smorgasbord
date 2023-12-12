use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{
    backend::{Key, Path, Write},
    widget_tree::Value,
};

use super::{
    widget::{self, Kind, Size},
    BuildResult,
};

#[derive(Clone, Debug)]
pub struct Simple {
    value: network_tables::Value,
}

impl Kind for Simple {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let widget = Paragraph::new(self.value.to_string());
        widget.render(area, buf);
    }

    fn prompt(&self) -> String {
        String::from("Simple widgets are constant")
    }

    fn update(&mut self, _path: &Path, _text: &str) -> Write {
        Write::default()
    }

    fn update_nt(&mut self, _key: &Key, value: &Value) {
        if let Value::Leaf(value) = &value {
            self.value = value.clone();
        }
    }

    fn reset(&mut self) {}

    fn is_finished(&self) -> bool {
        true
    }

    fn size(&self) -> Size {
        Size {
            width: 1,
            height: 1,
        }
    }

    fn clone_box(&self) -> Box<dyn Kind> {
        Box::new(self.clone())
    }
}

pub struct Builder;

impl widget::Builder for Builder {
    fn create_kind(&self, key: &Key, value: &Value) -> BuildResult {
        let Value::Leaf(value) = value else {
            return BuildResult::None;
        };

        if key.as_str().starts_with('.') {
            BuildResult::None
        } else {
            BuildResult::Complete(Box::new(Simple {
                value: value.clone(),
            }))
        }
    }
}
