use ratatui::{
	crossterm::event::{Event, KeyCode}, style::Stylize, text::Line, widgets::{Block, Paragraph, Widget, Wrap}
};

use crate::ui::modifier_magnitude;

pub struct LogsTab {
	// TODO more sophisticated scrolls? maybe scrollbar?
	scroll: u16,
}

impl super::Tab for LogsTab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event) {
		if let Event::Key(ev) = event {
			let modifier = modifier_magnitude(ev) as u16;
			match ev.code {
				KeyCode::Up => self.scroll -= modifier,
				KeyCode::Down => self.scroll += modifier,
				_ => {},
			}
		}
	}
}

impl super::Renderable for LogsTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_widget(LogsTabWidget(self.scroll), area);
	}
}

impl LogsTab {
	pub fn new() -> Self {
		Self { scroll: 0 }
	}
}





struct LogsTabWidget(u16);

impl Widget for LogsTabWidget {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		let container = Block::bordered()
			.title("app logs")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.dark_gray();

		Paragraph::new(
			crate::logger::all().into_iter().rev().collect::<Vec<String>>()
				.into_iter()
				.map(Line::from)
				.collect::<Vec<Line>>(),
		)
		.white()
		.block(container)
		.wrap(Wrap { trim: true })
		.scroll((self.0, 0))
		.render(area, buf);
	}
}
