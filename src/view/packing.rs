use std::{collections::HashMap, rc::Rc};

use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    widgets::StatefulWidget,
};

use crate::{
    backend::Path,
    state::packing::GridPosition,
    widgets::{self, Size, Widget},
};

pub struct View<'a> {
    pub size: Size,
    pub widgets: &'a HashMap<GridPosition, Widget>,
    pub titles: &'a HashMap<Path, GridPosition>,
}

pub struct State {
    pub cursor: GridPosition,
    pub selected: bool,
}

impl StatefulWidget for View<'_> {
    type State = State;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let rows_constraints =
            vec![Constraint::Ratio(1, u32::try_from(self.size.height).unwrap()); self.size.height];
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(rows_constraints)
            .split(area);

        let row_constraints =
            vec![Constraint::Ratio(1, u32::try_from(self.size.width).unwrap()); self.size.width];

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
            let start = row_layouts[position.y][position.x];
            let end = row_layouts[position.y + widget.size().height - 1]
                [position.x + widget.size().width - 1];
            let rect = start.union(end);

            let in_cursor = state.cursor.x >= position.x
                && state.cursor.x < position.x + widget.size().width
                && state.cursor.y >= position.y
                && state.cursor.y < position.y + widget.size().height;

            let mut state = if in_cursor {
                if state.selected {
                    widgets::State::Selected
                } else {
                    widgets::State::Highlighted
                }
            } else {
                widgets::State::Unhighlighted
            };

            widget.clone().render(rect, buf, &mut state);
        }
    }
}
