use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget as UIWidget,
    },
};

use crate::state::widget::{Widget, WidgetKind};

use super::table::Selectable;

pub struct WidgetState {
    is_selected: bool,
}

impl Selectable for WidgetState {
    fn select(&mut self, is_selected: bool) {
        self.is_selected = is_selected;
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self { is_selected: false }
    }
}

impl WidgetState {
    fn style(&self) -> Style {
        let color = if self.is_selected {
            Color::Magenta
        } else {
            Color::White
        };
        Style::default().fg(color)
    }
}

impl StatefulWidget for Widget {
    type State = WidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let title_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .style(state.style());
        let title_widget =
            Paragraph::new(Text::styled(self.title, Style::default().fg(Color::Red)))
                .block(title_block);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Min(self.value.height())])
            .split(area);

        title_widget.render(layout[0], buf);
        StatefulWidget::render(self.value, layout[1], buf, state);
    }
}

impl StatefulWidget for WidgetKind {
    type State = WidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .style(state.style());
        match self {
            WidgetKind::Simple { value } => {
                let text = value
                    .map(|value| value.to_string())
                    .unwrap_or(String::from(""));
                let widget = Paragraph::new(Text::styled(text, Style::default().fg(Color::Yellow)))
                    .block(block);
                widget.render(area, buf);
            }
            WidgetKind::Chooser {
                options,
                default,
                active,
            } => {
                let mut items = Vec::new();
                let mut index = None;

                for (i, option) in options.into_iter().enumerate() {
                    #[allow(clippy::if_same_then_else)]
                    if option == active {
                        index = Some(i);
                    } else if option == default && index.is_none() {
                        index = Some(i);
                    }
                    items.push(ListItem::new(option));
                }
                let widget = List::new(items)
                    .style(Style::default().fg(Color::Yellow))
                    .highlight_style(Style::default().fg(Color::LightBlue))
                    .block(block);
                let mut state = ListState::default();
                state.select(index);
                StatefulWidget::render(widget, area, buf, &mut state);
            }
        }
    }
}
