use ratatui::{
    prelude::{Alignment, Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget as UIWidget},
    Frame,
};

use crate::{backend::Backend, state::App};
use crate::{backend::Status, state::app::State};

use super::packing;

impl<B: Backend> App<B> {
    pub fn render(&self, f: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(3 * self.packing.size.height as u16),
                Constraint::Length(5),
            ])
            .split(f.size());

        self.render_title(chunks[0], f.buffer_mut());

        let mut cursor_state = packing::State {
            selected: match self.state {
                State::View => false,
                State::Edit(_) => true,
            },
            cursor: self.cursor,
        };

        let packing_view = self.packing.widget();

        f.render_stateful_widget(packing_view, chunks[1], &mut cursor_state);

        self.render_edit_window(chunks[2], f.buffer_mut());
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
                    .scroll((0, u16::try_from(scroll).unwrap()))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(edit.prompt.clone()),
                    );
                input.render(area, buf);
            }
        }
    }

    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(title_block.inner(area));

        let title = Paragraph::new("Smorgasbord");

        let status = self.network_table.status();

        title_block.render(area, buf);
        title.render(layout[0], buf);
        status.render(layout[1], buf);
    }
}

impl UIWidget for Status {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (color, text) = if self.is_connected {
            (Color::Green, "Connected")
        } else {
            (Color::Red, "Disconnected")
        };
        let widget = Paragraph::new(text)
            .style(Style::default().fg(color))
            .alignment(Alignment::Right);
        widget.render(area, buf);
    }
}
