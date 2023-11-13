use network_tables::Value;

use crate::{
    nt::Key,
    trie::{KeysRef, Trie},
    widgets::Widget,
};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

struct Grid {
    width: usize,
    height: usize,
    widgets: HashMap<GridPosition, Widget>,
}

impl Grid {
    fn add_widget(&mut self, position: GridPosition, widget: Widget) {
        self.widgets.insert(position, widget);
    }
}

pub struct ManagedGrid {
    grid: Grid,
    next_position: GridPosition,
}

impl ManagedGrid {
    pub fn add_widget(&mut self, widget: Widget) {
        if self.next_position.y < self.grid.height {
            self.grid.add_widget(self.next_position, widget);
        }
        self.next_position.x += 1;

        if self.next_position.x == self.grid.width {
            self.next_position.y += 1;
            self.next_position.x = 0;
        }
    }

    pub fn populate_from(&mut self, widgets: impl IntoIterator<Item = Widget>) {
        for new_widget in widgets {
            let mut add = true;
            for old_widget in self.grid.widgets.values_mut() {
                if old_widget.title == new_widget.title {
                    old_widget.value = new_widget.value.clone_box();
                    add = false;
                }
            }
            if add {
                self.add_widget(new_widget);
            }
        }
    }

    pub fn get_widgets(&self) -> impl IntoIterator<Item = (GridPosition, Widget)> {
        let mut widgets = Vec::new();

        let widget_id = move |position: GridPosition| position.x + position.y * self.grid.width;

        for (position, widget) in &self.grid.widgets {
            let index =
                widgets.binary_search_by_key(&widget_id(*position), |(pos, _)| widget_id(*pos));
            if let Err(index) = index {
                widgets.insert(index, (*position, widget.clone()));
            }
        }

        widgets
    }

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid {
                width,
                height,
                widgets: HashMap::default(),
            },
            next_position: GridPosition { x: 0, y: 0 },
        }
    }

    pub fn get_width(&self) -> usize {
        self.grid.width
    }

    pub fn get_height(&self) -> usize {
        self.grid.height
    }

    pub fn get_mut_widget(&mut self, index: &GridPosition) -> Option<&mut Widget> {
        self.grid.widgets.get_mut(index)
    }

    pub fn has_widget(&self, widget: &Widget) -> bool {
        for existing_widget in self.grid.widgets.values() {
            if existing_widget.title == widget.title {
                return true;
            }
        }
        false
    }

    pub fn update_widgets(&mut self, trie: &Trie<Key, Value>) {
        for widget in self.grid.widgets.values_mut() {
            let key_ref = KeysRef {
                first: &widget.title.first,
                rest: widget.title.rest.iter(),
            };
            let relevant_nt = trie.get_subtrie(key_ref).unwrap();
            widget.update_nt(relevant_nt);
        }
    }
}
