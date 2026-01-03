use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::{Constraint, Layout}, style::{Style, Stylize}, text::Line, widgets::{Block, List, ListState, Padding, Paragraph, Wrap}
};

use crate::sub::{self, provider::Queue};

pub struct PlayingTab {
	provider: sub::Provider,
	state: ListState,
}

impl super::Tab for PlayingTab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event) {
		if let Event::Key(ev) = event {
			let modifier = crate::ui::modifier_magnitude(ev) as usize;
			let idx = self.state.selected().unwrap_or_default();
			match ev.code {
				KeyCode::Char('s') => self.provider.shuffle_liked(),
				KeyCode::Up => self.state.select(Some(idx.saturating_sub(modifier))),
				KeyCode::Down => self.state.select(Some(idx.saturating_add(modifier))),
				KeyCode::Backspace => { self.provider.dequeue(idx); },
				KeyCode::Esc => self.state.select(None),
				KeyCode::Enter => {
					if let Some(s) = self.provider.get_at(idx) {
						self.provider.dequeue(idx);
						self.provider.enqueue_next(s);
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

impl super::Renderable for PlayingTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_stateful_widget(PlayingTabWidget(self.provider.current(), self.provider.view(), self.provider.index()), area, &mut self.state);
	}
}

impl PlayingTab {
	pub fn new(provider: sub::Provider) -> Self {
		Self { provider, state: ListState::default() }
	}
}




struct PlayingTabWidget(Option<submarine::data::Child>, Vec<submarine::data::Child>, usize);

impl ratatui::widgets::StatefulWidget for PlayingTabWidget {
	type State = ListState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
	where
		Self: Sized,
	{
		let [_, pad, _] = Layout::vertical([
			Constraint::Percentage(20),
			Constraint::Percentage(60),
			Constraint::Percentage(20),
		])
		.areas(area);
		let [main, queue] =
			Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]).areas(pad);
		let [_, content, _] = Layout::horizontal([
			Constraint::Percentage(20),
			Constraint::Percentage(60),
			Constraint::Percentage(20),
		])
		.areas(main);

		let up_next= self.1.into_iter().enumerate().map(|(i, s)| {
			let txt = format!("{} - {} ({})", s.title, s.artist.as_deref().unwrap_or_default(), s.album.as_deref().unwrap_or_default());
			if i == self.2 {
				Line::from(txt.gray().bold())
			} else {
				Line::from(txt.italic())
			}
		}).collect::<Vec<Line>>();

		let queue_box = Block::new().padding(Padding::uniform(content.height / 10));
		List::new(up_next)
			.block(queue_box)
			.scroll_padding(5)
			.highlight_spacing(ratatui::widgets::HighlightSpacing::Never)
			.highlight_style(Style::new().white().on_red())
			.dark_gray()
			.render(queue, buf, state);

		let border = Block::bordered()
			.padding(Padding::new(2, 2, content.height / 5, content.height / 5))
			.red();

		let info = if let Some(song) = self.0 {
			vec![
				Line::from(song.title.clone().red()),
				Line::from(song.artist.as_deref().unwrap_or("?").to_string().white()),
				Line::from(song.album.as_deref().unwrap_or("?").to_string().gray()),
				Line::from(""),
				Line::from(
					format!(
						"#{} - {}",
						song.track.unwrap_or_default(),
						song.year.unwrap_or_default()
					)
					.gray(),
				),
				Line::from(song.content_type.unwrap_or_default().dark_gray()),
			]
		} else {
			vec![]
		};

		{
			use ratatui::widgets::Widget;
			Paragraph::new(info)
				.block(border)
				.centered()
				.wrap(Wrap { trim: true })
				.render(content, buf);
		}
	}
}
