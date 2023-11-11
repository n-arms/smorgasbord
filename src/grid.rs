use network_tables::Value;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridPosition {
    x: usize,
    y: usize,
}

#[derive(Clone)]
pub struct Widget {
    pub table_id: String,
    pub value: Option<Value>,
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
        self.grid.add_widget(self.next_position, widget);
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
                if old_widget.table_id == new_widget.table_id {
                    old_widget.value = new_widget.value.clone();
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
}
