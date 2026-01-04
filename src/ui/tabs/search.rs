use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::{Constraint, Layout}, style::{Style, Stylize}, widgets::{Block, Paragraph, StatefulWidget, Table, TableState}
};

use crate::{sub::{self, provider::Queue}, ui::modifier_magnitude};

pub struct SearchTab {
	provider: sub::Provider,
	query: String,
	table: TableState,
	inserting: bool,
}

impl super::Tab for SearchTab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event) -> bool {
		if let Event::Key(ev) = event {
			let modifier = modifier_magnitude(ev) as usize;
			if self.inserting {
				match ev.code {
					KeyCode::Esc => self.inserting = false,
					KeyCode::Char(c) => self.query.push(c),
					KeyCode::Backspace => { self.query.pop(); },
					KeyCode::Enter => {
						self.provider.search(self.query.clone());
						self.inserting = false;
					},
					_ => {},
				}

				return true;

			} else {
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
					KeyCode::Char('=') => {
						let idx = self.table.selected().unwrap_or_default();
						if let Some(song) = self.provider.search_results().get(idx) {
							self.provider.enqueue_next(song.clone());
						}
					},
					KeyCode::Char('+') => {
						let idx = self.table.selected().unwrap_or_default();
						if let Some(song) = self.provider.search_results().get(idx) {
							self.provider.enqueue(song.clone());
						}
					},
					KeyCode::Esc => self.inserting = true,
					_ => {},
				}

				return false;

			}
		}

		false
	}
}

impl super::Renderable for SearchTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		frame.render_stateful_widget(
			SearchTabWidget(self.query.clone(), self.provider.search_results(), self.inserting),
			area,
			&mut self.table,
		);
	}
}

impl SearchTab {
	pub fn new(provider: sub::Provider) -> Self {
		Self { provider, query: String::new(), table: TableState::default(), inserting: false }
	}
}




struct SearchTabWidget(String, Vec<sub::Song>, bool);

impl StatefulWidget for SearchTabWidget {
	type State = TableState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut TableState)
	where
		Self: Sized,
	{
		let [query, results] = Layout::vertical([
			Constraint::Length(3),
			Constraint::Percentage(100)
		])
			.areas(area);

		let active_style = Style::new().gray();
		let inactive_style = Style::new().dark_gray();

		let query_border = Block::bordered()
			.title("search")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.style(if self.2 { active_style } else { inactive_style });

		{
			use ratatui::widgets::Widget;
			Paragraph::new(self.0)
				.block(query_border)
				.white()
				.render(query, buf);
		}

		let results_border = Block::bordered()
			.title("results")
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.style(if self.2 { inactive_style } else { active_style });

		let (heade,rows, widths) = crate::ext::tabularize(self.1);

		Table::new(rows, widths)
			.block(results_border)
			.header(heade)
			.row_highlight_style(Style::new().black().on_red().bold())
			.render(results, buf, state);
	}
}

