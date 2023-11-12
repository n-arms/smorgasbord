use network_tables::Value;
use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget as UIWidget,
    },
};

#[derive(Clone, Debug)]
pub struct Widget {
    pub title: String,
    pub value: WidgetKind,
}

#[derive(Clone, Debug)]
pub enum WidgetKind {
    Simple {
        value: Option<Value>,
    },
    Chooser {
        options: Vec<String>,
        default: String,
        active: String,
    },
}
impl WidgetKind {
    fn height(&self) -> u16 {
        let raw_height = match self {
            WidgetKind::Simple { .. } => 1,
            WidgetKind::Chooser { options, .. } => options.len() as u16,
        };
        raw_height + 1
    }
}

impl UIWidget for Widget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .style(Style::default());
        let title_widget =
            Paragraph::new(Text::styled(self.title, Style::default().fg(Color::Red)))
                .block(title_block);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Min(self.value.height())])
            .split(area);

        title_widget.render(layout[0], buf);
        self.value.render(layout[1], buf);
    }
}

impl UIWidget for WidgetKind {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .style(Style::default().fg(Color::White));
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
