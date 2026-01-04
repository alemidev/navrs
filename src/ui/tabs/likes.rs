use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::{Constraint, Margin}, style::{Style, Stylize}, widgets::{Block, Padding, Row, Scrollbar, ScrollbarState, StatefulWidget, Table, TableState}
};

use crate::{sub::{self, provider::Queue}, ui::modifier_magnitude};

pub struct LikesTab {
	provider: sub::Provider,
	state: LikesTabState,
}

#[derive(Default)]
struct LikesTabState {
	table: TableState,
	scroll: ScrollbarState,
}

impl LikesTab {
	pub fn new(provider: sub::Provider) -> Self {
		Self { provider, state: LikesTabState::default() }
	}
}

impl super::Tab for LikesTab {
	fn handle_input(&mut self, event: &Event) -> bool {
		if let Event::Key(ev) = event {
			let modifier = modifier_magnitude(ev) as usize;
			let selected = self.state.table.selected().unwrap_or_default();

			match ev.code {
				KeyCode::Up => {
					self.state.scroll = self.state.scroll.position(selected);
					self.state.table.select(Some(selected.saturating_sub(modifier)));
				},
				KeyCode::Down => {
					self.state.scroll = self.state.scroll.position(selected);
					self.state.table.select(Some(selected.saturating_add(modifier)));
				},
				KeyCode::Char('?') => {
					self.provider.shuffle_liked();
				},
				KeyCode::Char('=') => {
					if let Some(song) = self.provider.likes().get(selected) {
						self.provider.enqueue_next(song.clone());
					}
				},
				KeyCode::Char('+') => {
					if let Some(song) = self.provider.likes().get(selected) {
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
		frame.render_stateful_widget(LikesTabWidget(self.provider.likes()), area, &mut self.state);
	}
}




struct LikesTabWidget(Vec<submarine::data::Child>);

impl StatefulWidget for LikesTabWidget {
	type State = LikesTabState;

	fn render(
		self,
		area: ratatui::prelude::Rect,
		buf: &mut ratatui::prelude::Buffer,
		state: &mut Self::State,
	) where
		Self: Sized,
	{
		state.scroll = state.scroll.content_length(self.0.len());

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
			Constraint::Min(5),
		];

		Table::new(rows, widths)
			.block(block)
			.header(
				Row::new(["TITLE", "ARTIST", "ALBUM", "PLAY COUNT"])
					.white()
					.on_black()
					.bold(),
			)
			.row_highlight_style(Style::new().black().on_red().bold())
			.render(area, buf, &mut state.table);

		let scroll_area = area.inner(Margin {
			// using an inner vertical margin of 1 unit makes the scrollbar inside the block
			vertical: 1,
			horizontal: 1,
		});

		Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
			.style(Style::new().dark_gray())
			.render(scroll_area, buf, &mut state.scroll);
	}
}

fn song_into_row<'a>(song: submarine::data::Child) -> Row<'a> {
	Row::new([
		song.title.clone(),
		song.artist.clone().unwrap_or_default(),
		song.album.clone().unwrap_or_default(),
		song.play_count.unwrap_or_default().to_string(),
	])
	.white()
}
