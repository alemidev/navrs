use ratatui::{
	DefaultTerminal, Frame,
	crossterm::event::{self, Event, KeyCode},
	layout::{Constraint, Layout},
	style::{Style, Stylize},
	widgets::{Block, Paragraph, Tabs},
};

use crate::{
	music::{MusicProvider, SubsonicProvider},
	ui::tabs::{AppTabs, LikesTab, PlayingTab, Renderable, library::LibraryTab, logs::LogsTab, queue::QueueTab},
};

const SUBTUI: &str = r#"    _   /__/_   .
  _\/_//_// /_// "#;

pub struct App {
	provider: SubsonicProvider,

	tab: AppTabs,

	playing_tab: PlayingTab,
	likes_tab: LikesTab,
	library_tab: LibraryTab,
	queue_tab: QueueTab,
	logs_tab: LogsTab,


	flip_flop: bool,
}

impl App {
	pub fn new(provider: SubsonicProvider) -> Self {
		Self {
			playing_tab: PlayingTab::new(provider.clone()),
			likes_tab: LikesTab::new(provider.clone()),
			library_tab: LibraryTab::new(),
			queue_tab: QueueTab::new(provider.clone()),
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
			AppTabs::Library => &mut self.library_tab,
			AppTabs::Queue => &mut self.queue_tab,
			AppTabs::Logs => &mut self.logs_tab,
		}
	}

	pub fn run(mut self, mut term: DefaultTerminal) -> std::io::Result<()> {
		self.provider.likes(); // preload them

		loop {
			self.flip_flop = !self.flip_flop;

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
					self.tab().handle_input(&ev);
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
								KeyCode::Char('3') => self.tab = AppTabs::Library,
								KeyCode::Char('4') => self.tab = AppTabs::Queue,
								KeyCode::Char('5') => self.tab = AppTabs::Logs,
								KeyCode::Char(' ') => {
									self.provider.toggle();
								}
								KeyCode::Char('n') | KeyCode::Char('$') => { self.provider.next(); },
								KeyCode::Char('r') | KeyCode::Char('^') => { self.provider.restart(); }
								KeyCode::Right => { self.provider.skip(0.01 * modifier as f32); },
								KeyCode::Left => { self.provider.skip(-0.01 * modifier as f32); },
								KeyCode::Tab => self.tab = self.tab.next(),
								_ => {}
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

		let tab_layout = Layout::horizontal([Constraint::Percentage(100), Constraint::Min(52)]);
		let [title, tabbar] = tab_layout.areas(tabs);

		let tabs_border = Block::bordered().gray();
		let t = Tabs::new(AppTabs::titles())
			.block(tabs_border)
			.highlight_style(Style::new().red().bold())
			.padding(" ", " ")
			.divider(" | ")
			.select(Some(self.tab.idx()))
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

		self.tab().render(frame, content);

		let widget = crate::ui::playbar::Playbar(self.provider.clone());
		frame.render_widget(widget, playbar);
	}
}
