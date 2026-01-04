pub mod playing;
pub mod likes;
pub mod search;
pub mod library;
pub mod logs;

pub use likes::LikesTab;
pub use playing::PlayingTab;

pub trait Tab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event) -> bool;
}

pub trait Renderable : Tab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AppTabs {
	Playing,
	Likes,
	Search,
	Library,
	Logs,
}

impl AppTabs {
	pub fn next(self) -> Self {
		match self {
			Self::Playing => Self::Likes,
			Self::Likes => Self::Search,
			Self::Search => Self::Library,
			Self::Library => Self::Logs,
			Self::Logs => Self::Playing,
		}
	}

	pub fn idx(&self) -> usize {
		match self {
			Self::Playing => 0,
			Self::Likes => 1,
			Self::Search => 2,
			Self::Library => 3,
			Self::Logs => 4,
		}
	}

	pub const fn titles() -> [&'static str; 5] {
		[
			"playing",
			"likes",
			"search",
			"library",
			"logs",
		]
	}
}
