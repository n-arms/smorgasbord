use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget as UIWidget},
    Frame,
};

use crate::state::app::State;
use crate::state::App;
use crate::view::table::Table;

use super::widget::WidgetState;

impl App {
    pub fn render(&self, f: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(5),
            ])
            .split(f.size());

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let title = Paragraph::new(Text::styled(
            "Smorgasbord",
            Style::default().fg(Color::Green),
        ))
        .block(title_block);

        f.render_widget(title, chunks[0]);

        let table = Table::new(
            self.grid.get_widgets(),
            self.grid.get_width(),
            self.grid.get_height(),
            self.cursor,
            self.cursor_state(),
        );

        f.render_widget(table, chunks[1]);

        self.render_edit_window(chunks[2], f.buffer_mut());
    }

    fn cursor_state(&self) -> WidgetState {
        if let State::Edit(edit) = &self.state {
            edit.state.clone()
        } else {
            WidgetState {
                is_selected: true,
                is_finished: true,
            }
        }
    }

    fn render_edit_window(&self, area: Rect, buf: &mut Buffer) {
        match &self.state {
            State::View => Paragraph::new("Smorgasbord")
                .block(Block::new().borders(Borders::ALL))
                .render(area, buf),
            State::Edit(edit) => {
                let width = area.width.max(3) - 3;
                let scroll = edit.text_field.visual_scroll(width as usize);
                let input = Paragraph::new(edit.text_field.value())
                    .scroll((0, scroll as u16))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(edit.prompt.clone()),
                    );
                input.render(area, buf);
            }
        }
    }
}
