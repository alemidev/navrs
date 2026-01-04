use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::Constraint, style::{Style, Stylize}, widgets::{Block, Padding, Row, StatefulWidget, Table, TableState}
};

use crate::{sub::{self, provider::Queue}, ui::modifier_magnitude};

pub struct LikesTab {
	table: TableState,
	provider: sub::Provider,
}

impl LikesTab {
	pub fn new(provider: sub::Provider) -> Self {
		Self { table: TableState::default(), provider }
	}
}

impl super::Tab for LikesTab {
	fn handle_input(&mut self, event: &Event) -> bool {
		if let Event::Key(ev) = event {
			let modifier = modifier_magnitude(ev) as usize;

			match ev.code {
				KeyCode::Up => {
					self.table.select(Some(
						self.table
							.selected()
							.unwrap_or_default()
							.saturating_sub(modifier),
					));
				},
				KeyCode::Down => {
					self.table.select(Some(
						self.table
							.selected()
							.unwrap_or_default()
							.saturating_add(modifier),
					));
				},
				KeyCode::Char('?') => {
					self.provider.shuffle_liked();
				},
				KeyCode::Char('=') => {
					let idx = self.table.selected().unwrap_or_default();
					if let Some(song) = self.provider.likes().get(idx) {
						self.provider.enqueue_next(song.clone());
					}
				},
				KeyCode::Char('+') => {
					let idx = self.table.selected().unwrap_or_default();
					if let Some(song) = self.provider.likes().get(idx) {
						self.provider.enqueue(song.clone());
					}
				},
				KeyCode::Char('r') => {
					self.provider.refresh_likes();
				},
				_ => {},
			}
		}

		false
	}
}

impl super::Renderable for LikesTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_stateful_widget(LikesTabWidget(self.provider.likes()), area, &mut self.table);
	}
}




struct LikesTabWidget(Vec<submarine::data::Child>);

impl StatefulWidget for LikesTabWidget {
	type State = TableState;

	fn render(
		self,
		area: ratatui::prelude::Rect,
		buf: &mut ratatui::prelude::Buffer,
		state: &mut Self::State,
	) where
		Self: Sized,
	{
		let block = Block::bordered()
			.padding(Padding::uniform(2))
			.title("liked songs")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.dark_gray();

		let rows = self.0.into_iter().map(song_into_row).collect::<Vec<Row>>();
		let widths = [
			Constraint::Min(30),
			Constraint::Min(15),
			Constraint::Min(15),
		];

		Table::new(rows, widths)
			.block(block)
			.header(
				Row::new(["TITLE", "ARTIST", "ALBUM"])
					.white()
					.on_black()
					.bold(),
			)
			.row_highlight_style(Style::new().black().on_red().bold())
			.render(area, buf, state);
	}
}

fn song_into_row<'a>(song: submarine::data::Child) -> Row<'a> {
	Row::new([
		song.title.clone(),
		song.artist.clone().unwrap_or_default(),
		song.album.clone().unwrap_or_default(),
	])
	.white()
}
