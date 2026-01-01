use ratatui::{
	style::Stylize, text::Line, widgets::{Block, Paragraph, Widget}
};

pub struct QueueTab(pub Vec<sunk::song::Song>);

impl Widget for QueueTab {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		let up_next= self.0.into_iter().enumerate().map(|(i, s)| {
			let txt = format!("{} - {} ({})", s.title, s.artist.as_deref().unwrap_or_default(), s.album.as_deref().unwrap_or_default());
			if i == 0 {
				Line::from(txt.gray().bold())
			} else {
				Line::from(txt.italic())
			}
		}).collect::<Vec<Line>>();

		let block = Block::bordered()
			.title("queue")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.dark_gray();

		Paragraph::new(up_next)
			.block(block)
			.right_aligned()
			.dark_gray()
			.render(area, buf);
	}
}
