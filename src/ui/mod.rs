pub mod app;
pub mod playbar;
pub mod tabs;

pub use app::App;

pub fn modifier_magnitude(ev: &ratatui::crossterm::event::KeyEvent) -> f64 {
	if ev.modifiers.contains(ratatui::crossterm::event::KeyModifiers::SHIFT) {
		return 5.;
	}

	if ev.modifiers.contains(ratatui::crossterm::event::KeyModifiers::ALT) {
		return 25.;
	}

	1.
}

const BORDER_STYLE: ratatui::widgets::BorderType = if cfg!(feature = "rounded") {
	ratatui::widgets::BorderType::Rounded
} else {
	ratatui::widgets::BorderType::Plain
};
