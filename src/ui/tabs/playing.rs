use ratatui::{
	crossterm::event::{Event, KeyCode}, layout::{Constraint, Layout, Size}, style::{Style, Stylize}, text::Line, widgets::{Block, List, ListState, Padding, Paragraph, Wrap}
};
use ratatui_image::{Resize, picker::Picker, protocol::Protocol};

use crate::sub::{self, cache::Cache};

pub struct PlayingTab {
	provider: sub::Provider,
	state: ListState,
	picker: Picker,
	last: sub::Id,
	image: Option<Protocol>,
	show_info: bool,
}

impl super::Tab for PlayingTab {
	fn handle_input(&mut self, event: &ratatui::crossterm::event::Event) -> bool {
		if let Event::Key(ev) = event {
			let modifier = crate::ui::modifier_magnitude(ev) as usize;
			let idx = self.state.selected().unwrap_or(self.provider.queue.index());
			match ev.code {
				KeyCode::Char('i') => self.show_info = !self.show_info,
				KeyCode::Char('s') => self.provider.shuffle_liked(),
				KeyCode::Up => self.state.select(Some(idx.saturating_sub(modifier))),
				KeyCode::Down => self.state.select(Some(idx.saturating_add(modifier))),
				KeyCode::Esc => self.state.select(None),
				KeyCode::Backspace => { self.provider.queue.dequeue(idx); },
				KeyCode::Char('D') => self.provider.reset(Vec::new()),
				KeyCode::Char('=') => {
					if let Some(s) = self.provider.queue.get(idx) {
						self.provider.queue.dequeue(idx);
						self.provider.queue.enqueue_next(s);
					}
				},
				KeyCode::Enter => {
					self.provider.queue.set_index(idx);
					if let Some(song) = self.provider.queue.current() {
						self.provider.play(song.id);
					}
				}
				KeyCode::PageUp => {
					if let Some(s) = self.provider.queue.get(idx) {
						self.provider.queue.dequeue(idx);
						self.provider.queue.enqueue_at(idx.saturating_sub(1), s);
						self.state.select_previous();
					}
				},
				KeyCode::PageDown => {
					if let Some(s) = self.provider.queue.get(idx) {
						self.provider.queue.dequeue(idx);
						self.provider.queue.enqueue_at(idx.saturating_add(1), s);
						self.state.select_next();
					}
				},
				_ => {},
			}
		}

		false
	}
}

const SCROLL_PADDING: usize = 3;

impl super::Renderable for PlayingTab {
	fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
		if self.state.selected().is_none() && self.provider.queue.index() > SCROLL_PADDING {
			*self.state.offset_mut() = self.provider.queue.index() - SCROLL_PADDING;
		}

		let curr = self.provider.queue.current();

		if let Some(ref c) = curr && let Some(data) = sub::cache::data().lookup(&c.id) {
			if c.id != self.last {
				self.image = None;

				if let Some(i_data) = data.image {
					let mut img_builder = image::ImageReader::new(std::io::Cursor::new(i_data.data));
					if let Some(fmt) = i_data.format {
						img_builder.set_format(fmt);
					}
					
					match img_builder.decode() {
						Err(e) => log::error!("could not decode cover image: {e}"),
						Ok(dyn_img) => {
							let size = Size {
								width: area.width * 6 / 10,
								height: area.height * 6 / 10
							};
							match self.picker.new_protocol(dyn_img, size, Resize::Scale(None)) {
								Ok(img) => self.image = Some(img),
								Err(e) => log::error!("error creating image protocol: {e}"),
							}
						}
					}
				}
			}

			self.last = c.id.clone();
		}

		frame.render_stateful_widget(
			PlayingTabWidget(self.provider.queue.current(), self.provider.queue.view(), self.provider.queue.index(), self.image.clone(), self.show_info),
			area,
			&mut self.state
		);
	}
}

impl PlayingTab {
	pub fn new(provider: sub::Provider) -> Self {
		Self {
			provider,
			state: ListState::default(),
			picker: Picker::from_query_stdio().unwrap_or_else(|err| {
				log::error!("error querying for terminal image protocol (defaulting to halfblocks) - {err}");
				Picker::halfblocks()
			}),
			last: "".to_string(),
			image: None,
			show_info: false,
		}
	}
}




struct PlayingTabWidget(Option<submarine::data::Child>, Vec<submarine::data::Child>, usize, Option<Protocol>, bool);

impl ratatui::widgets::StatefulWidget for PlayingTabWidget {
	type State = ListState;

	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
	where
		Self: Sized,
	{
		let [_, pad, _] = Layout::vertical([
			Constraint::Percentage(20),
			Constraint::Percentage(60),
			Constraint::Percentage(20),
		])
		.areas(area);
		let [main, queue] =
			Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]).areas(pad);
		let [_, content, _] = Layout::horizontal([
			Constraint::Percentage(20),
			Constraint::Percentage(60),
			Constraint::Percentage(20),
		])
		.areas(main);

		let up_next= self.1.into_iter().enumerate().map(|(i, s)| {
			let txt = format!("{} - {} ({})", s.title, s.artist.as_deref().unwrap_or_default(), s.album.as_deref().unwrap_or_default());
			if i == self.2 {
				Line::from(txt.gray().bold())
			} else {
				Line::from(txt.italic())
			}
		}).collect::<Vec<Line>>();

		let queue_box = Block::new().padding(Padding::uniform(content.height / 10));
		List::new(up_next)
			.block(queue_box)
			.scroll_padding(SCROLL_PADDING)
			.highlight_spacing(ratatui::widgets::HighlightSpacing::Never)
			.highlight_style(Style::new().white().on_red())
			.dark_gray()
			.render(queue, buf, state);

		let border = Block::bordered()
			.padding(Padding::new(2, 2, content.height / 5, content.height / 5))
			.red();

		if self.4 {
			let lines = if let Some(song) = self.0 {
				vec![
					Line::from(song.title.clone().red()),
					Line::from(song.artist.as_deref().unwrap_or("?").to_string().white()),
					Line::from(song.album.as_deref().unwrap_or("?").to_string().gray()),
					Line::from(""),
					Line::from(
						format!(
							"#{} - {} plays",
							song.track.unwrap_or_default(),
							song.play_count.unwrap_or_default()
						)
						.gray(),
					),
					Line::from(format!("{} ({})", song.year.unwrap_or_default(), song.genre.unwrap_or_default()).dark_gray()),
					Line::from(""),
					Line::from(format!("{} @{}kbps", song.content_type.unwrap_or_default(), song.bit_rate.unwrap_or_default()).gray()),
					Line::from(if song.starred.is_some() { "starred".red() } else { "".dark_gray() }),
				]
			} else {
				vec![]
			};

			use ratatui::widgets::Widget;
			Paragraph::new(lines)
				.block(border)
				.centered()
				.wrap(Wrap { trim: true })
				.render(content, buf);

		} else {

			// TODO would be nice to put a box around this image but it 
			//let block = Block::bordered().red();
			//let inner_content = block.inner(sqr_content);

			use ratatui::widgets::Widget;
			//block.render(sqr_content, buf);

			if let Some(protocol) = self.3 {
				ratatui_image::Image::new(&protocol)
					.allow_clipping(true)
					.render(content, buf);
			}
		}

	}
}
