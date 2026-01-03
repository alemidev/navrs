use ratatui::{
	crossterm::event::{Event, KeyCode}, style::{Style, Stylize}, text::Line, widgets::{Block, List, ListState, StatefulWidget}
};

use crate::music::{MusicProvider, SubsonicProvider};

pub struct QueueTab {
	provider: SubsonicProvider,
	state: ListState,
}

impl super::Tab for QueueTab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event) {
		if let Event::Key(ev) = event {
			let modifier = crate::ui::modifier_magnitude(ev) as usize;
			let idx = self.state.selected().unwrap_or_default();
			match ev.code {
				KeyCode::Up => self.state.select(Some(idx.saturating_sub(modifier))),
				KeyCode::Down => self.state.select(Some(idx.saturating_add(modifier))),
				KeyCode::Backspace => { self.provider.pop_queue(idx); },
				KeyCode::Enter => {
					if let Some(s) = self.provider.pop_queue(idx) {
						self.provider.play_next(s);
					}
				},
				// TODO move up and down in queue
				// KeyCode::PageUp => {},
				// KeyCode::PageDown => {},
				_ => {},
			}

		}
	}
}

impl super::Renderable for QueueTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_stateful_widget(QueueTabWidget(self.provider.queue()), area, &mut self.state);
	}
}

impl QueueTab {
	pub fn new(provider: SubsonicProvider) -> Self {
		Self { provider, state: ListState::default() }
	}
}




struct QueueTabWidget(Vec<submarine::data::Child>);

impl StatefulWidget for QueueTabWidget {
	type State = ListState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
	where
		Self: Sized,
	{
		let up_next= self.0
			.into_iter()
			.enumerate()
			.map(|(i, s)| {
				let txt = format!("{} - {} ({})", s.title, s.artist.as_deref().unwrap_or_default(), s.album.as_deref().unwrap_or_default());
				if i == 0 {
					Line::from(txt.bold().white())
				} else {
					Line::from(txt.italic().gray())
				}
			})
			.collect::<Vec<Line>>();

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
