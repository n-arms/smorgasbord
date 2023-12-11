use std::collections::HashMap;

use crate::{
    backend::Path,
    view::packing,
    widget_tree::Tree,
    widgets::{Size, Widget},
};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

pub struct Packing {
    pub size: Size,
    pub widgets: HashMap<GridPosition, Path>,
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

    pub fn add(&mut self, path: Path, widgets: &Tree) {
        let widget = widgets.get(&path).unwrap();
        if self.titles.get(&widget.title).is_some() {
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
                self.widgets.insert(position, path);
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

    pub fn get_mut_widget<'a>(
        &self,
        position: GridPosition,
        tree: &'a mut Tree,
    ) -> Option<&'a mut Widget> {
        let mut pos = None;
        'find: for row in (0..=position.y).rev() {
            for col in (0..=position.x).rev() {
                let current = GridPosition { x: col, y: row };
                let path = self.widgets.get(&current);
                let widget = path.map(|path| tree.get(path).unwrap());
                if widget.is_some() {
                    let size = widget.unwrap().size();
                    if row + size.height > position.y && col + size.width > position.x {
                        pos = Some(current);
                        break 'find;
                    }
                }
            }
        }
        let path = self.widgets.get(&pos?).unwrap();
        tree.get_mut(path)
    }

    pub fn widget<'a>(&'a self, tree: &'a Tree) -> packing::View<'a> {
        let mut widgets = HashMap::new();
        for (position, path) in &self.widgets {
            widgets.insert(*position, tree.get(path).unwrap());
        }
        packing::View {
            size: self.size,
            widgets,
            titles: &self.titles,
        }
    }
}
