use ratatui::{
    prelude::{Alignment, Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget as UIWidget},
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
                Constraint::Min(3 * u16::try_from(self.packing.size.height).unwrap()),
                Constraint::Length(5),
            ])
            .split(f.size());

        self.render_title(chunks[0], f.buffer_mut());

        self.render_grid(chunks[1], f.buffer_mut());

        self.render_edit_window(chunks[2], f.buffer_mut());
    }

    fn render_grid(&self, area: Rect, buf: &mut Buffer) {
        /*
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(30 * self.packing.size.width as u16),
                Constraint::Min(20),
            ])
            .split(area);
        let mut cursor_state = packing::State {
            selected: match self.state {
                State::View => false,
                State::Edit(_) => true,
            },
            cursor: self.cursor,
        };

        let packing_view = self.packing.widget();

        packing_view.render(chunks[0], buf, &mut cursor_state);

        self.tab_selector.view().render(chunks[1], buf);*/

        let mut cursor_state = packing::State {
            selected: match self.state {
                State::View => false,
                State::Edit(_) => true,
            },
            cursor: self.cursor,
        };

        let packing_view = self.packing.widget(&self.widget_tree);

        packing_view.render(area, buf, &mut cursor_state);
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
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
            ])
            .split(title_block.inner(area));

        let title = Paragraph::new("Smorgasbord");

        let elapsed = Paragraph::new(format!("{:?}", self.start_time.elapsed()));

        let status = self.network_table.status();

        title_block.render(area, buf);
        title.render(layout[0], buf);
        elapsed.render(layout[1], buf);
        status.render(layout[2], buf);
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
