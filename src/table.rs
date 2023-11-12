use std::{collections::HashMap, rc::Rc};

use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use crate::grid::GridPosition;

#[derive(Debug)]
pub struct Table<I> {
    widgets: I,
    width: usize,
    height: usize,
}

impl<I> Table<I> {
    pub fn new(widgets: I, width: usize, height: usize) -> Self {
        Table {
            widgets,
            width,
            height,
        }
    }
}

impl<T: Widget, I: IntoIterator<Item = (GridPosition, T)>> Widget for Table<I> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_constraints = vec![Constraint::Ratio(1, self.height as u32); self.height];
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(rows_constraints)
            .split(area);

        let row_constraints = vec![Constraint::Ratio(1, self.width as u32); self.width];

        let row_layouts: Vec<Rc<[Rect]>> = rows_layout
            .iter()
            .map(|row| {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(row_constraints.clone())
                    .split(*row)
            })
            .collect();

        for (position, widget) in self.widgets {
            widget.render(row_layouts[position.y][position.x], buf);
        }
    }
}
