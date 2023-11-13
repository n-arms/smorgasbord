use std::rc::Rc;

use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    widgets::{StatefulWidget, Widget as UIWidget},
};

use crate::state::grid::GridPosition;
use crate::widgets::State;

pub trait Selectable {
    fn select(&mut self, is_selected: bool);
}

#[derive(Debug)]
pub struct Table<I> {
    widgets: I,
    width: usize,
    height: usize,
    cursor: GridPosition,
    cursor_state: State,
}

impl<I> Table<I> {
    pub fn new(
        widgets: I,
        width: usize,
        height: usize,
        cursor: GridPosition,
        cursor_state: State,
    ) -> Self {
        Table {
            widgets,
            width,
            height,
            cursor,
            cursor_state,
        }
    }
}

impl<T: StatefulWidget<State = State>, I: IntoIterator<Item = (GridPosition, T)>> UIWidget
    for Table<I>
{
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
            let mut state = if position == self.cursor {
                self.cursor_state
            } else {
                State::Unhighlighted
            };
            widget.render(row_layouts[position.y][position.x], buf, &mut state);
        }
    }
}
