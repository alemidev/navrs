use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::Margin, style::{Style, Stylize}, text::Line, widgets::{Block, Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget, Wrap}
};

use crate::ui::modifier_magnitude;

pub struct LogsTab {
	scroll: usize,
	scroll_state: ScrollbarState,
}

impl super::Tab for LogsTab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event) -> bool {
		if let Event::Key(ev) = event {
			let modifier = modifier_magnitude(ev) as usize;
			match ev.code {
				KeyCode::Up => {
					self.scroll = self.scroll.saturating_sub(modifier);
					self.scroll_state = self.scroll_state.position(self.scroll);
				},
				KeyCode::Down => {
					self.scroll = self.scroll.saturating_add(modifier);
					self.scroll_state = self.scroll_state.position(self.scroll);
				},
				KeyCode::Esc => {
					self.scroll = 0;
					self.scroll_state.first();
				},
				_ => {},
			}
		}

		false
	}
}

impl super::Renderable for LogsTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_stateful_widget(LogsTabWidget(self.scroll), area, &mut self.scroll_state);
	}
}

impl LogsTab {
	pub fn new() -> Self {
		Self {
			scroll: 0,
			scroll_state: ScrollbarState::default(),
		}
	}
}





struct LogsTabWidget(usize);

impl StatefulWidget for LogsTabWidget {
	type State = ScrollbarState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
	where
		Self: Sized,
	{
		let container = Block::bordered()
			.title("app logs")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.border_type(ratatui::widgets::BorderType::Rounded)
			.dark_gray();

		let logs = crate::logger::all().into_iter().rev().collect::<Vec<String>>()
			.into_iter()
			.map(Line::from)
			.collect::<Vec<Line>>();

		*state = state.content_length(logs.len());

		Paragraph::new(logs)
			.white()
			.wrap(Wrap { trim: false })
			.block(container)
			.scroll((self.0 as u16, 0))
			.render(area, buf);

		let scroll_area = area.inner(Margin {
			// using an inner vertical margin of 1 unit makes the scrollbar inside the block
			vertical: 1,
			horizontal: 1,
		});

		Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
			.style(Style::new().dark_gray())
			.render(scroll_area, buf, state);
	}
}
