use ratatui::{layout::Constraint, style::Stylize, widgets::Row};

use crate::sub;

pub fn tabularize<'a>(songs: Vec<sub::Song>) -> (Row<'a>, Vec<Row<'a>>, [Constraint; 3]) {
	let widths = [
		Constraint::Min(30),
		Constraint::Min(15),
		Constraint::Min(15),
	];

	let header = Row::new(["TITLE", "ARTIST", "ALBUM"])
		.white()
		.on_black()
		.bold();

	let mut rows = Vec::new();

	for song in songs {
		rows.push(
			Row::new([
				song.title,
				song.artist.unwrap_or_default(),
				song.album.unwrap_or_default(),
			])
				.white()
		);
	}

	(header, rows, widths)
}
