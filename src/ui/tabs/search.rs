use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::{Constraint, Layout, Margin}, style::{Style, Stylize}, widgets::{Block, Padding, Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Table, TableState}
};

use crate::{sub::{self, provider::Queue}, ui::modifier_magnitude};

pub struct SearchTab {
	provider: sub::Provider,
	query: String,
	state: SearchTabState,
	inserting: bool,
}

#[derive(Default)]
struct SearchTabState {
	table: TableState,
	scroll: ScrollbarState,
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
				let idx = self.state.table.selected().unwrap_or_default();
				match ev.code {
					KeyCode::Up => {
						self.state.scroll = self.state.scroll.position(idx);
						self.state.table.select(Some(idx.saturating_sub(modifier)));
					},
					KeyCode::Down => {
						self.state.scroll = self.state.scroll.position(idx);
						self.state.table.select(Some(idx.saturating_add(modifier)));
					},
					KeyCode::Char('=') => {
						if let Some(song) = self.provider.search_results().get(idx) {
							self.provider.enqueue_next(song.clone());
						}
					},
					KeyCode::Char('+') => {
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
			&mut self.state,
		);
	}
}

impl SearchTab {
	pub fn new(provider: sub::Provider) -> Self {
		Self { provider, query: String::new(), state: SearchTabState::default(), inserting: false }
	}
}




struct SearchTabWidget(String, Vec<sub::Song>, bool);

impl StatefulWidget for SearchTabWidget {
	type State = SearchTabState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
	where
		Self: Sized,
	{
		state.scroll = state.scroll.content_length(self.1.len());

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
			.border_type(crate::ui::BORDER_STYLE)
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
			.padding(Padding::uniform(2))
			.title_alignment(ratatui::layout::HorizontalAlignment::Right)
			.border_type(crate::ui::BORDER_STYLE)
			.style(if self.2 { inactive_style } else { active_style });

		let (heade,rows, widths) = crate::ext::tabularize(self.1);

		Table::new(rows, widths)
			.block(results_border)
			.header(heade)
			.row_highlight_style(Style::new().black().on_red().bold())
			.render(results, buf, &mut state.table);

		let scroll_area = results.inner(Margin {
			// using an inner vertical margin of 1 unit makes the scrollbar inside the block
			vertical: 1,
			horizontal: 1,
		});

		Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
			.style(if self.2 { inactive_style } else { active_style })
			.render(scroll_area, buf, &mut state.scroll);
	}
}

