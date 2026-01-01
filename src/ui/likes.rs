use ratatui::{layout::Constraint, style::{Style, Stylize}, widgets::{Block, Padding, Row, ScrollbarState, StatefulWidget, Table, TableState}};


pub struct LikesTab(pub Vec<sunk::song::Song>);

#[derive(Default)]
pub struct LikesTabState {
	pub scroll: ScrollbarState,
	pub table: TableState,
}

impl StatefulWidget for LikesTab {
	type State = LikesTabState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
		where Self: Sized
	{
		let block = Block::bordered()
			.padding(Padding::uniform(2))
			.title("liked songs")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.dark_gray();

		let rows = self.0.into_iter().map(song_into_row).collect::<Vec<Row>>();
		let widths = [Constraint::Min(30), Constraint::Min(15), Constraint::Min(15)];

		Table::new(rows, widths)
			.block(block)
			.header(Row::new(["TITLE", "ARTIST", "ALBUM"]).white().on_black().bold())
			.row_highlight_style(Style::new().black().on_red().bold())
			.render(area, buf, &mut state.table);
	}
}

fn song_into_row<'a>(song: sunk::song::Song) -> Row<'a> {
	Row::new([
		song.title.clone(),
		song.artist.as_ref().map(|x| x.clone()).unwrap_or_default(),
		song.album.as_ref().map(|x| x.clone()).unwrap_or_default(),
	]).white()
}
