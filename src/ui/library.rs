use ratatui::{
	style::Stylize,
	widgets::{Block, Paragraph, Widget},
};

pub struct LibraryTab;

impl Widget for LibraryTab {
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
