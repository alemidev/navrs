use ratatui::{
	style::{Style, Stylize}, text::Line, widgets::{Block, List, ListState, StatefulWidget}
};

pub struct QueueTab(pub Vec<sunk::song::Song>);

impl StatefulWidget for QueueTab {
	type State = ListState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
	where
		Self: Sized,
	{
		let up_next= self.0.into_iter().map(|s| {
			let txt = format!("{} - {} ({})", s.title, s.artist.as_deref().unwrap_or_default(), s.album.as_deref().unwrap_or_default());
			Line::from(txt.italic())
		}).collect::<Vec<Line>>();

		let block = Block::bordered()
			.title("queue")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.dark_gray();

		List::new(up_next)
			.block(block)
			.scroll_padding(5)
			.highlight_spacing(ratatui::widgets::HighlightSpacing::WhenSelected)
			.highlight_style(Style::new().white().on_red())
			.highlight_symbol(" * ")
			.render(area, buf, state);
	}
}
