use ratatui::{
	layout::{Constraint, Layout},
	style::{Color, Stylize},
	widgets::{Block, Gauge, Paragraph, Widget, Wrap},
};

use crate::music::{MusicProvider, SubsonicProvider};

pub struct Playbar<T: MusicProvider<f32>>(pub T);

impl Widget for Playbar<SubsonicProvider> {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		let running = self.0.running();
		let block = Block::bordered()
			.title(if running {
				" |> playing "
			} else {
				" || paused "
			})
			.title_alignment(ratatui::layout::HorizontalAlignment::Center)
			.red();
		let inner_playbar = block.inner(area);
		block.render(area, buf);

		let [up, down] = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
			.areas(inner_playbar);
		let [artist, title, album] = Layout::horizontal([
			Constraint::Percentage(33),
			Constraint::Percentage(34),
			Constraint::Percentage(33),
		])
		.areas(up);

		Paragraph::new(
			"artist: ".dark_gray()
				+ self
					.0
					.song()
					.and_then(|x| x.artist.clone())
					.unwrap_or("".to_string())
					.red(),
		)
		.wrap(Wrap { trim: true })
		.render(artist, buf);

		Paragraph::new(
			"album: ".dark_gray()
				+ self
					.0
					.song()
					.as_ref()
					.and_then(|x| x.album.clone())
					.unwrap_or("".to_string())
					.red(),
		)
		.right_aligned()
		.wrap(Wrap { trim: true })
		.render(album, buf);

		Paragraph::new(
			"title: ".dark_gray()
				+ self
					.0
					.song()
					.as_ref()
					.map(|x| x.title.clone())
					.unwrap_or("".to_string())
					.red(),
		)
		.centered()
		.wrap(Wrap { trim: true })
		.render(title, buf);

		Gauge::default()
			.gauge_style(Color::Red)
			.ratio(self.0.progress() as f64)
			.label("")
			.render(down, buf);
	}
}
