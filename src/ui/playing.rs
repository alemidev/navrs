use ratatui::{layout::{Constraint, Layout}, style::Stylize, text::Line, widgets::{Block, Padding, Paragraph, Widget, Wrap}};


pub struct PlayingTab(pub Option<sunk::song::Song>);

impl Widget for PlayingTab {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
		where Self: Sized
	{
		let [_, pad, _] = Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)]).areas(area);
		let [main, queue] = Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]).areas(pad);
		let [_, content, _] = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)]).areas(main);

		let up_next = vec![
			Line::from("next song"),
			Line::from("even further song"),
			Line::from("wow so many songs"),
		];

		let queue_box = Block::new().padding(Padding::uniform(2));
		Paragraph::new(up_next)
			.block(queue_box)
			.right_aligned()
			.dark_gray()
			.render(queue, buf);

		let border = Block::bordered().padding(Padding::vertical(5)).red();

		let info = if let Some(song) = self.0 {
			vec![
				Line::from(song.title.clone().red()),
				Line::from(song.artist.as_deref().unwrap_or("?").to_string().white()),
				Line::from(song.album.as_deref().unwrap_or("?").to_string().gray()),
				Line::from(""),
				Line::from(format!("#{} - {}", song.track.unwrap_or_default(), song.year.unwrap_or_default()).gray()),
				Line::from(song.content_type.clone().dark_gray()),
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
