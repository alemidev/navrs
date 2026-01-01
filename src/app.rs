use ratatui::{
	DefaultTerminal, Frame,
	crossterm::event::{self, Event, KeyCode, KeyModifiers},
	layout::{Constraint, Layout},
	style::{Style, Stylize},
	widgets::{Block, Paragraph, Tabs},
};

use crate::{
	music::{MusicProvider, SubsonicProvider},
	ui::likes::LikesTabState,
};

const SUBTUI: &str = r#"    _   /__/_   .
  _\/_//_// /_// "#;

pub struct App {
	pub provider: SubsonicProvider,
	pub scroll: u16,
	pub tab: usize,

	tab_state: LikesTabState,
	flip_flop: bool,
}

impl App {
	pub fn new(provider: SubsonicProvider) -> Self {
		Self {
			provider,
			scroll: 0,
			tab: 0,
			tab_state: LikesTabState::default(),
			flip_flop: false,
		}
	}

	pub fn run(mut self, mut term: DefaultTerminal) -> sunk::Result<()> {
		self.provider.likes(); // preload them

		loop {
			self.flip_flop = !self.flip_flop;

			let _frame = term.draw(|frame| self.draw(frame))?;

			match event::poll(std::time::Duration::from_millis(200)) {
				Err(e) => log::error!("err polling event: {e}"),
				Ok(false) => {}
				Ok(true) => match event::read() {
					Err(e) => log::error!("err reading event: {e}"),
					Ok(ev) => match ev {
						Event::FocusGained => {}
						Event::FocusLost => {}
						Event::Paste(_) => {}
						Event::Resize(_, _) => {}
						Event::Mouse(_mouse_event) => {}
						Event::Key(key_event) => {
							let modifier = if key_event.modifiers.contains(KeyModifiers::SHIFT) {
								5
							} else {
								1
							};
							match key_event.code {
								KeyCode::Char('q') => return Ok(()),
								KeyCode::Char('n') => { self.provider.next(); },
								KeyCode::Char('1') => {
									self.tab = 0;
								}
								KeyCode::Char('2') => {
									self.tab = 1;
								}
								KeyCode::Char('3') => {
									self.tab = 2;
								}
								KeyCode::Char('4') => {
									self.tab = 3;
								}
								KeyCode::Char('5') => {
									self.tab = 4;
								}
								KeyCode::Char(' ') => {
									self.provider.toggle();
								}
								KeyCode::Char('?') => {
									self.provider.random_song()?;
								}
								KeyCode::PageUp => {
									self.scroll = self.scroll.saturating_add(1);
								}
								KeyCode::PageDown => {
									self.scroll = self.scroll.saturating_sub(1);
								}
								KeyCode::Right => { self.provider.skip(0.01 * modifier as f32); },
								KeyCode::Left => { self.provider.skip(-0.01 * modifier as f32); },
								KeyCode::Up => {
									self.tab_state.table.select(Some(
										self.tab_state
											.table
											.selected()
											.unwrap_or_default()
											.saturating_sub(modifier),
									));
								}
								KeyCode::Down => {
									self.tab_state.table.select(Some(
										self.tab_state
											.table
											.selected()
											.unwrap_or_default()
											.saturating_add(modifier),
									));
								}
								KeyCode::Tab => {
									self.tab = (self.tab + 1) % 5;
								}
								KeyCode::Char('+') => {
									if self.tab == 1 {
										let idx = self.tab_state.table.selected().unwrap_or_default();
										if let Some(song) = self.provider.likes().get(idx) {
											self.provider.enqueue(vec![song.clone()]);
										}
									}
								},
								KeyCode::Enter => {
									match self.tab {
										0 => self.provider.shuffle_liked(),
										1 => {
											let idx = self.tab_state.table.selected().unwrap_or_default();
											if let Some(song) = self.provider.likes().get(idx) {
												self.provider.play(song.clone());
											}
										},
										_ => {},
									}
								}
								_ => {}
							}
						}
					},
				},
			}
		}
	}

	fn draw(&mut self, frame: &mut Frame) {
		let area = frame.area();

		let vertical = Layout::vertical([
			Constraint::Length(3),
			Constraint::Min(0),
			Constraint::Length(4),
		]);
		let [tabs, content, playbar] = vertical.areas(area);

		let tab_layout = Layout::horizontal([Constraint::Percentage(100), Constraint::Min(52)]);
		let [title, tabbar] = tab_layout.areas(tabs);

		let tabs_border = Block::bordered().gray();
		let t = Tabs::new(["playing", "likes", "library", "queue", "log"])
			.block(tabs_border)
			.highlight_style(Style::new().red().bold())
			.padding(" ", " ")
			.divider(" | ")
			.select(Some(self.tab))
			.gray();
		frame.render_widget(t, tabbar);

		let mut subtui = Paragraph::new(SUBTUI);
		if self.provider.working() {
			if self.flip_flop {
				subtui = subtui.dark_gray();
			} else {
				subtui = subtui.red();
			}
		} else {
			subtui = subtui.red().bold();
		};
		frame.render_widget(subtui, title);

		match self.tab {
			0 => frame.render_widget(
				crate::ui::playing::PlayingTab(self.provider.current_song(), self.provider.queue()),
				content,
			),
			1 => frame.render_stateful_widget(
				crate::ui::likes::LikesTab(self.provider.likes()),
				content,
				&mut self.tab_state,
			),
			2 => frame.render_widget(crate::ui::library::LibraryTab, content),
			3 => frame.render_widget(crate::ui::queue::QueueTab, content),
			4 => frame.render_widget(crate::ui::logs::LogsTab(self.scroll), content),
			_ => frame.render_widget(Block::bordered().title("wrong tab index").red(), content),
		}

		let widget = crate::ui::playbar::Playbar(self.provider.clone());
		frame.render_widget(widget, playbar);
	}
}
