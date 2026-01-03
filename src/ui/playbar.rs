use ratatui::{
	layout::{Constraint, Layout},
	style::{Color, Stylize},
	widgets::{Block, Gauge, Paragraph, Widget, Wrap},
};

use crate::sub::{self, provider::{Buffer, Player, Queue}};

pub struct Playbar(pub sub::Provider);

impl Widget for Playbar {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		let running = !self.0.paused();
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

		let song = self.0.current();

		Paragraph::new(
			"artist: ".dark_gray()
				+ song
					.as_ref()
					.and_then(|x| x.artist.clone())
					.unwrap_or("".to_string())
					.red(),
		)
		.wrap(Wrap { trim: true })
		.render(artist, buf);

		Paragraph::new(
			"album: ".dark_gray()
				+ song
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
				+ song
					.as_ref()
					.map(|x| x.title.clone())
					.unwrap_or("".to_string())
					.red(),
		)
		.centered()
		.wrap(Wrap { trim: true })
		.render(title, buf);
		
		let [now, bar, end] = Layout::horizontal([Constraint::Min(5), Constraint::Percentage(100), Constraint::Min(5)]).areas(down);

		let duration = song.and_then(|s| s.duration).unwrap_or_default();
		let current_time = (duration as f32 * self.0.progress()) as i32;
		let remaining = duration - current_time;

		Paragraph::new(format!("{}.{:02}", current_time / 60, current_time % 60))
			.dark_gray()
			.render(now, buf);

		Paragraph::new(format!("{}.{:02}", duration / 60, duration % 60))
			.right_aligned()
			.dark_gray()
			.render(end, buf);

		Gauge::default()
			.gauge_style(Color::Red)
			.ratio(self.0.progress() as f64)
			.label(format!("{}.{:02}", remaining / 60, remaining % 60))
			.render(bar, buf);
	}
}
