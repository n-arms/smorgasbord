use std::collections::{HashMap, HashSet};

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
    pub titles: HashSet<Path>,
    pub occupied: Vec<bool>,
}

impl Packing {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            widgets: Default::default(),
            titles: Default::default(),
            occupied: vec![false; size.width * size.height],
        }
    }

    fn is_occupied(&self, position: GridPosition) -> bool {
        self.occupied[position.x + position.y * self.size.width]
    }

    fn set_occupied(&mut self, position: GridPosition, is_occupied: bool) {
        self.occupied[position.x + position.y * self.size.width] = is_occupied;
    }

    fn fits(&self, start: GridPosition, size: Size) -> bool {
        for row in start.y..start.y + size.height {
            for col in start.x..start.x + size.width {
                if self.is_occupied(GridPosition { x: col, y: row }) {
                    return false;
                }
            }
        }
        true
    }

    fn insert(&mut self, position: GridPosition, size: Size, path: Path) {
        for row in position.y..position.y + size.height {
            for col in position.x..position.x + size.width {
                self.set_occupied(GridPosition { x: col, y: row }, true);
            }
        }
        self.titles.insert(path.clone());
        self.widgets.insert(position, path);
    }

    fn add_unchecked(&mut self, path: Path, size: Size) {
        for start_row in 0..self.size.height {
            for start_col in 0..self.size.width {
                let position = GridPosition {
                    x: start_col,
                    y: start_row,
                };

                if self.fits(position, size) {
                    self.insert(position, size, path);
                    return;
                }
            }
        }
    }

    pub fn add(&mut self, path: Path, widgets: &Tree) {
        if self.titles.get(&path).is_none() {
            let size = widgets.get(&path).unwrap().size();
            self.add_unchecked(path, size);
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

    pub fn clear(&mut self) {
        self.widgets.clear();
        self.titles.clear();
        self.occupied.clear();
    }

    pub fn add_all(&mut self, mut all_widgets: Vec<Path>, widget_tree: &Tree) {
        all_widgets.retain(|path| !self.titles.contains(&path));
        all_widgets.sort_by_key(|path| widget_tree.get(path).map(|widget| widget.size().area()));
        all_widgets.reverse();

        for widget in all_widgets {
            let size = widget_tree.get(&widget).unwrap().size();
            self.add_unchecked(widget, size);
        }
    }
}
