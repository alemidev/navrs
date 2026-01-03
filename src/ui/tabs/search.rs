use ratatui::{
	style::Stylize,
	widgets::{Block, Paragraph, Widget},
};

pub struct SearchTab;

impl super::Tab for SearchTab {
	fn handle_input(&mut self, _event: &ratatui::crossterm::event::Event) {}
}

impl super::Renderable for SearchTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_widget(SearchTabWidget, area);
	}
}

impl SearchTab {
	pub fn new() -> Self {
		Self
	}
}




struct SearchTabWidget;

impl Widget for SearchTabWidget {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		let block = Block::bordered()
			.title("search")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.dark_gray();

		Paragraph::new("TODO").block(block).centered().render(
			area.centered_vertically(ratatui::layout::Constraint::Percentage(100)),
			buf,
		);
	}
}
