use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::{Constraint, Layout}, style::Stylize, text::Line, widgets::{Block, Padding, Paragraph, Widget, Wrap}
};

use crate::music::{MusicProvider, SubsonicProvider};

pub struct PlayingTab {
	provider: SubsonicProvider,
}

impl super::Tab for PlayingTab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event) {
		if let Event::Key(ev) = event {
			match ev.code {
				KeyCode::Enter => self.provider.shuffle_liked(),
				_ => {},
			}
		}
	}
}

impl super::Renderable for PlayingTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_widget(PlayingTabWidget(self.provider.current_song(), self.provider.queue()), area);
	}
}

impl PlayingTab {
	pub fn new(provider: SubsonicProvider) -> Self {
		Self { provider }
	}
}




struct PlayingTabWidget(Option<submarine::data::Child>, Vec<submarine::data::Child>);

impl Widget for PlayingTabWidget {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
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
			if i == 0 {
				Line::from(txt.gray().bold())
			} else {
				Line::from(txt.italic())
			}
		}).collect::<Vec<Line>>();

		let queue_box = Block::new().padding(Padding::uniform(content.height / 10));
		Paragraph::new(up_next)
			.block(queue_box)
			.right_aligned()
			.dark_gray()
			.render(queue, buf);

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

		Paragraph::new(info)
			.block(border)
			.centered()
			.wrap(Wrap { trim: true })
			.render(content, buf);
	}
}
