use itertools::Itertools;
use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::{Constraint, Layout}, style::Style, widgets::{Block, List, ListItem, ListState, StatefulWidget}
};

use crate::{sub, ui::modifier_magnitude};

pub struct LibraryTab {
	provider: sub::Provider,
	selected: Area,
	state: LibraryState,

	cached_albums: Vec<sub::Album>,
	cached_songs: Vec<sub::Song>,
}

impl super::Tab for LibraryTab {
	fn handle_input(&mut self, event: &Event) -> bool {
		if let Event::Key(ev) = event {
			let modifier = modifier_magnitude(ev) as usize;
			match ev.code {
				KeyCode::Esc => {
					match self.selected {
						Area::Songs => {
							self.selected = Area::Albums;
							self.state.songs.selected_mut().take();
						},
						Area::Albums => {
							self.selected = Area::Artists;
							self.state.albums.selected_mut().take();
						},
						Area::Artists => {},
					}
				},
				KeyCode::Enter => {
					match self.selected {
						Area::Artists => {
							let idx = self.state.artists.selected().unwrap_or_default();
							if let Some(artist) = self.provider.artists.get().get(idx) {
								self.cached_songs.clear();
								self.cached_albums = self.provider.albums
									.get()
									.iter()
									.filter(|a| a.artist_id.as_ref().map(|x| *x == artist.id).unwrap_or_default())
									.sorted_by_key(|a| a.year)
									.cloned()
									.collect();
								self.selected = Area::Albums;
								self.state.albums.selected_mut().take();
								self.state.songs.selected_mut().take();
							}
						},
						Area::Albums => {
							let idx = self.state.albums.selected().unwrap_or_default();
							if let Some(album) = self.cached_albums.get(idx) {
								self.cached_songs = self.provider.songs
									.get()
									.iter()
									.filter(|s| s.album_id.as_ref().map(|x| *x == album.id).unwrap_or_default())
									.sorted_by_key(|s| s.track)
									.cloned()
									.collect();
								self.selected = Area::Songs;
								self.state.songs.selected_mut().take();
							}
						},
						Area::Songs => {
							if let Some(sel) = self.state.songs.selected()
								&& let Some(song) = self.cached_songs.get(sel)
							{
								self.provider.queue.enqueue_next(song.clone());
								if self.provider.queue.len() > 1 {
									self.provider.go_next();
								}
							}
						},
					}
				},
				KeyCode::Up => {
					match self.selected {
						Area::Songs => {
							self.state.songs.select(Some(
								self.state.songs
									.selected()
									.unwrap_or_default()
									.saturating_sub(modifier),
							))
						},
						Area::Albums => {
							self.state.albums.select(Some(
								self.state.albums
									.selected()
									.unwrap_or_default()
									.saturating_sub(modifier),
							))
						},
						Area::Artists => {
							self.state.artists.select(Some(
								self.state.artists
									.selected()
									.unwrap_or_default()
									.saturating_sub(modifier),
							));
						},
					}
				},
				KeyCode::Down => {
					match self.selected {
						Area::Songs => {
							self.state.songs.select(Some(
								self.state.songs
									.selected()
									.unwrap_or_default()
									.saturating_add(modifier),
							))
						},
						Area::Albums => {
							self.state.albums.select(Some(
								self.state.albums
									.selected()
									.unwrap_or_default()
									.saturating_add(modifier),
							))
						},
						Area::Artists => {
							self.state.artists.select(Some(
								self.state.artists
									.selected()
									.unwrap_or_default()
									.saturating_add(modifier),
							));
						},
					}
				},
				KeyCode::Char('=') => {
					match self.selected {
						Area::Artists => {},
						Area::Songs => {},
						Area::Albums => {
							if let Some(sel) = self.state.albums.selected()
								&& let Some(album) = self.cached_albums.get(sel)
							{
								let songs = self.provider.songs
									.get()
									.iter()
									.filter(|s| s.album_id.as_ref().map(|x| *x == album.id).unwrap_or_default())
									.sorted_by_key(|s| s.track)
									.cloned()
									.collect::<Vec<sub::Song>>();
								if let Some(first) = songs.first().cloned() {
									self.provider.play(first.id);
								}
								self.provider.reset(songs);
							}
						},
					}
				},
				_ => {},
			}
		}

		false
	}
}

impl super::Renderable for LibraryTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_stateful_widget(
			LibraryTabWidget(self.selected, self.provider.artists.get(), self.cached_albums.clone(), self.cached_songs.clone()),
			area,
			&mut self.state,
		);
	}
}

impl LibraryTab {
	pub fn new(provider: sub::Provider) -> Self {
		Self {
			provider,
			selected: Area::Artists,
			state: LibraryState::default(),
			cached_songs: Vec::new(),
			cached_albums: Vec::new(),
		}
	}
}

#[derive(Default)]
struct LibraryState {
	artists: ListState,
	albums: ListState,
	songs: ListState,
}

#[derive(Clone, Copy)]
enum Area {
	Artists,
	Albums,
	Songs,
}



const SCROLL_PADDING: usize = 3;

struct LibraryTabWidget(Area, Vec<sub::Artist>, Vec<sub::Album>, Vec<sub::Song>);

impl StatefulWidget for LibraryTabWidget {
	type State = LibraryState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut LibraryState)
	where
		Self: Sized,
	{
		let [artists, other] = Layout::horizontal([
			Constraint::Min(20),
			Constraint::Percentage(100),
		])
			.areas(area);

		let [albums, songs] = Layout::vertical([
			Constraint::Percentage(60),
			Constraint::Percentage(40),
		])
			.areas(other);

		let focused = Style::new().gray();
		let unfocused = Style::new().dark_gray();

		let artists_block = Block::bordered()
			.title("artists")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.border_type(crate::ui::BORDER_STYLE)
			.style(if matches!(self.0, Area::Artists) { focused } else { unfocused });

		let artists_data = self.1.into_iter().map(|x| ListItem::new(x.name)).collect::<Vec<ListItem>>();

		List::new(artists_data)
			.block(artists_block)
			.scroll_padding(SCROLL_PADDING)
			.highlight_spacing(ratatui::widgets::HighlightSpacing::WhenSelected)
			.highlight_style(Style::new().white().on_red())
			.style(if matches!(self.0, Area::Artists) { focused } else { unfocused })
			.render(artists, buf, &mut state.artists);

		let albums_block = Block::bordered()
			.title("albums")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.border_type(crate::ui::BORDER_STYLE)
			.style(if matches!(self.0, Area::Albums) { focused } else { unfocused });

		let albums_data = self.2
			.into_iter()
			.map(|x| ListItem::new(format!("{} - {}", x.year.unwrap_or_default(), x.name)))
			.collect::<Vec<ListItem>>();

		List::new(albums_data)
			.block(albums_block)
			.scroll_padding(SCROLL_PADDING)
			.highlight_spacing(ratatui::widgets::HighlightSpacing::WhenSelected)
			.highlight_style(Style::new().white().on_red())
			.style(if matches!(self.0, Area::Albums) { focused } else { unfocused })
			.render(albums, buf, &mut state.albums);

		let songs_block = Block::bordered()
			.title("songs")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.border_type(crate::ui::BORDER_STYLE)
			.style(if matches!(self.0, Area::Songs) { focused } else { unfocused });

		let songs_data = self.3
			.into_iter()
			.map(|x| ListItem::new(format!("#{:02} {}", x.track.unwrap_or_default(), x.title)))
			.collect::<Vec<ListItem>>();

		List::new(songs_data)
			.block(songs_block)
			.scroll_padding(SCROLL_PADDING)
			.highlight_spacing(ratatui::widgets::HighlightSpacing::WhenSelected)
			.highlight_style(Style::new().white().on_red())
			.style(if matches!(self.0, Area::Songs) { focused } else { unfocused })
			.render(songs, buf, &mut state.songs);
	}
}
