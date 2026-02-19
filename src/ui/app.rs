use ratatui::{
	DefaultTerminal, Frame,
	crossterm::event::{self, Event, KeyCode},
	layout::{Constraint, Layout},
	style::{Style, Stylize},
	widgets::{Block, Paragraph, Tabs},
};

use crate::{
	audio::api::{Player, Buffer}, sub, ui::tabs::{AppTabs, LikesTab, PlayingTab, Renderable, library::LibraryTab, logs::LogsTab, search::SearchTab}
};

const NAVRS: &str = r#"                 __   __  
 |\ |  /\  \  / |__) /__` 
 | \| /~~\  \/  |  \ .__/ "#;

pub struct App {
	provider: sub::Provider,

	tab: AppTabs,

	playing_tab: PlayingTab,
	likes_tab: LikesTab,
	search_tab: SearchTab,
	library_tab: LibraryTab,
	logs_tab: LogsTab,


	flip_flop: bool,
}

impl App {
	pub fn new(provider: sub::Provider) -> Self {
		Self {
			playing_tab: PlayingTab::new(provider.clone()),
			likes_tab: LikesTab::new(provider.clone()),
			search_tab: SearchTab::new(provider.clone()),
			library_tab: LibraryTab::new(provider.clone()),
			logs_tab: LogsTab::new(),

			tab: AppTabs::Playing,

			flip_flop: false,
			provider,
		}
	}

	pub fn tab(&mut self) -> &mut dyn Renderable {
		match self.tab {
			AppTabs::Playing => &mut self.playing_tab,
			AppTabs::Likes => &mut self.likes_tab,
			AppTabs::Search => &mut self.search_tab,
			AppTabs::Library => &mut self.library_tab,
			AppTabs::Logs => &mut self.logs_tab,
		}
	}

	pub fn run(mut self, mut term: DefaultTerminal) -> std::io::Result<()> {
		self.provider.refresh_likes();
		self.provider.refresh_library();

		loop {
			self.flip_flop = !self.flip_flop;

			if self.provider.player.progress() >= 1. {
				self.provider.go_next();
			}

			let _frame = term.draw(|frame| self.draw(frame))?;

			if self.poll_events() {
				break
			}
		}

		Ok(())
	}

	fn poll_events(&mut self) -> bool {
		match event::poll(std::time::Duration::from_millis(200)) {
			Err(e) => log::error!("err polling event: {e}"),
			Ok(false) => {}
			Ok(true) => match event::read() {
				Err(e) => log::error!("err reading event: {e}"),
				Ok(ev) => {
					if !self.tab().handle_input(&ev) {
						match ev {
							Event::FocusGained => {}
							Event::FocusLost => {}
							Event::Paste(_) => {}
							Event::Resize(_, _) => {}
							Event::Mouse(_mouse_event) => {}
							Event::Key(key_event) => {
								let modifier = crate::ui::modifier_magnitude(&key_event);
								match key_event.code {
									KeyCode::Char('q') => return true,
									KeyCode::Char('1') => self.tab = AppTabs::Playing,
									KeyCode::Char('2') => self.tab = AppTabs::Likes,
									KeyCode::Char('3') => self.tab = AppTabs::Search,
									KeyCode::Char('4') => self.tab = AppTabs::Library,
									KeyCode::Char('5') => self.tab = AppTabs::Logs,
									KeyCode::Char(' ') => self.provider.player.play_pause(),
									KeyCode::Char('n') | KeyCode::Char('$') => { self.provider.go_next(); },
									KeyCode::Char('b') | KeyCode::Char('^') => { self.provider.go_previous(); }
									KeyCode::Right => { self.provider.player.skip(0.01 * modifier as f32); },
									KeyCode::Left => { self.provider.player.skip(-0.01 * modifier as f32); },
									KeyCode::Tab => self.tab = self.tab.next(),
									_ => {}
								}
							}
						}
					}
				},
			},
		}
		false
	}

	fn draw(&mut self, frame: &mut Frame) {
		let area = frame.area();

		let vertical = Layout::vertical([
			Constraint::Length(3),
			Constraint::Min(0),
			Constraint::Length(4),
		]);
		let [tabs, content, playbar] = vertical.areas(area);

		let tab_layout = Layout::horizontal([Constraint::Percentage(100), Constraint::Min(53)]);
		let [title, tabbar] = tab_layout.areas(tabs);

		let tabs_border = Block::bordered()
			.border_type(crate::ui::BORDER_STYLE)
			.gray();
		let t = Tabs::new(AppTabs::titles())
			.block(tabs_border)
			.highlight_style(Style::new().red().bold())
			.padding(" ", " ")
			.divider(" | ")
			.select(Some(self.tab.idx()))
			.gray();
		frame.render_widget(t, tabbar);

		let mut navrs = Paragraph::new(NAVRS);
		if self.provider.working.get() {
			if self.flip_flop {
				navrs = navrs.dark_gray();
			} else {
				navrs = navrs.red();
			}
		} else {
			navrs = navrs.red().bold();
		};
		frame.render_widget(navrs, title);

		self.tab().render(frame, content);

		let widget = crate::ui::playbar::Playbar(self.provider.clone());
		frame.render_widget(widget, playbar);
	}
}
