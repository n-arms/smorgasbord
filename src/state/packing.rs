use std::collections::HashMap;

use crate::{
    backend::Path,
    view::packing,
    widgets::{Size, Widget},
};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

pub struct Packing {
    pub size: Size,
    pub widgets: HashMap<GridPosition, Widget>,
    pub titles: HashMap<Path, GridPosition>,
}

impl Packing {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            widgets: HashMap::new(),
            titles: HashMap::new(),
        }
    }

    pub fn add(&mut self, widget: Widget) {
        if let Some(position) = self.titles.get(&widget.title) {
            self.widgets.insert(*position, widget);
            return;
        }
        for start_row in 0..self.size.height {
            'finding: for start_col in 0..self.size.width {
                let size = widget.size();

                for row in start_row..start_row + size.height {
                    for col in start_col..start_col + size.width {
                        if self.widgets.contains_key(&GridPosition { x: col, y: row }) {
                            continue 'finding;
                        }
                    }
                }

                let position = GridPosition {
                    x: start_col,
                    y: start_row,
                };

                self.titles.insert(widget.title.clone(), position);
                self.widgets.insert(position, widget);
                return;
            }
        }
    }

    pub fn get_width(&self) -> usize {
        self.size.width
    }

    pub fn get_height(&self) -> usize {
        self.size.height
    }

    pub fn get_mut_widget(&mut self, position: GridPosition) -> Option<&mut Widget> {
        self.widgets.get_mut(&position)
    }

    pub fn widget(&self) -> packing::View {
        packing::View {
            size: self.size,
            widgets: &self.widgets,
            titles: &self.titles,
        }
    }
}
