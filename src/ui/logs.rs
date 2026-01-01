use ratatui::{
	style::Stylize,
	text::Line,
	widgets::{Block, Paragraph, Widget, Wrap},
};

pub struct LogsTab(pub u16);

impl Widget for LogsTab {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		let container = Block::bordered()
			.title("app logs")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.dark_gray();

		Paragraph::new(
			crate::logger::all()
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
