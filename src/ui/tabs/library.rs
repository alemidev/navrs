use ratatui::{
	style::Stylize,
	widgets::{Block, Paragraph, Widget},
};

pub struct LibraryTab;

impl super::Tab for LibraryTab {
	fn handle_input(&mut self, _event: &ratatui::crossterm::event::Event) {}
}

impl super::Renderable for LibraryTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_widget(LibraryTabWidget, area);
	}
}

impl LibraryTab {
	pub fn new() -> Self {
		Self
	}
}




struct LibraryTabWidget;

impl Widget for LibraryTabWidget {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		let block = Block::bordered()
			.title("library")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.dark_gray();

		Paragraph::new("TODO").block(block).centered().render(
			area.centered_vertically(ratatui::layout::Constraint::Percentage(100)),
			buf,
		);
	}
}
