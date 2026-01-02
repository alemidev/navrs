pub mod playing;
pub mod likes;
pub mod library;
pub mod queue;
pub mod logs;

pub use likes::LikesTab;
pub use playing::PlayingTab;

pub trait Tab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event);
}

pub trait Renderable : Tab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AppTabs {
	Playing,
	Likes,
	Library,
	Queue,
	Logs,
}

impl AppTabs {
	pub fn next(self) -> Self {
		match self {
			Self::Playing => Self::Likes,
			Self::Likes => Self::Library,
			Self::Library => Self::Queue,
			Self::Queue => Self::Logs,
			Self::Logs => Self::Playing,
		}
	}

	pub fn idx(&self) -> usize {
		match self {
			Self::Playing => 0,
			Self::Likes => 1,
			Self::Library => 2,
			Self::Queue => 3,
			Self::Logs => 4,
		}
	}

	pub const fn titles() -> [&'static str; 5] {
		[
			"playing",
			"likes",
			"library",
			"queue",
			"logs",
		]
	}
}
