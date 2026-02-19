use mpris_server::{
	LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, RootInterface, Time, TrackId, Volume, zbus::{Result, fdo}
};

use crate::{audio::api::{Buffer, Player}};

fn not_implemented<T>() -> fdo::Result<T> {
	Err(fdo::Error::NotSupported("not implemented".to_string()))
}

// TODO would be cool to impl these on the worker

impl RootInterface for crate::sub::Provider {
	async fn identity(&self) -> fdo::Result<String> {
		Ok("navrs".into())
	}

	async fn raise(&self) -> fdo::Result<()> { not_implemented() }
	async fn can_raise(&self) -> fdo::Result<bool> { Ok(false) }

	async fn quit(&self) -> fdo::Result<()> { not_implemented() }
	async fn can_quit(&self) -> fdo::Result<bool> { Ok(false) }

	async fn fullscreen(&self) -> fdo::Result<bool> { Ok(false) }
	async fn set_fullscreen(&self, _fullscreen:bool) -> Result<()> { Ok(()) }
	async fn can_set_fullscreen(&self) -> fdo::Result<bool> { Ok(false) }

	async fn has_track_list(&self) -> fdo::Result<bool> { Ok(false) } // TODO can do this!
	async fn desktop_entry(&self) -> fdo::Result<String> { Ok("navrs".to_string()) }

	async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> { Ok(Vec::new()) }
	async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> { Ok(Vec::new()) }
}

impl PlayerInterface for crate::sub::Provider {
	async fn set_volume(&self, _volume: Volume) -> Result<()> { Ok(()) } // TODO need volume...

	async fn metadata(&self) -> fdo::Result<Metadata> {
		if let Some(song) = self.queue.current() {
			Ok(
				Metadata::builder()
					.title(song.title)
					.artist([song.artist.unwrap_or_default()])
					.album(song.album.unwrap_or_default())
					.length(Time::from_secs(song.duration.unwrap_or_default() as i64))
					.build()
			)
		} else {
			Ok(Metadata::new())
		}
	}

	async fn next(&self) -> fdo::Result<()> {
		self.go_next();
		Ok(())
	}

	async fn previous(&self) -> fdo::Result<()> {
		self.go_previous();
		Ok(())
	}

	async fn pause(&self) -> fdo::Result<()> {
		self.player.pause();
		self.update_mpris();
		Ok(())
	}

	async fn play_pause(&self) -> fdo::Result<()> {
		self.player.play_pause();
		self.update_mpris();
		Ok(())
	}

	async fn stop(&self) -> fdo::Result<()> {
		self.player.pause();
		self.update_mpris();
		Ok(())
	}

	async fn play(&self) -> fdo::Result<()> {
		self.player.resume();
		self.update_mpris();
		Ok(())
	}

	async fn seek(&self, offset: Time) -> fdo::Result<()> {
		if let Some(s) = self.queue.current() && let Some(d) = s.duration {
			self.player.seek(d as f32 / offset.as_secs() as f32);
		}
		self.update_mpris();
		Ok(())
	}

	async fn set_position(&self, _track_id: TrackId, _position: Time) -> fdo::Result<()> {
		not_implemented()
	}

	async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
		not_implemented()
	}

	async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
		if self.player.paused() {
			Ok(PlaybackStatus::Paused)
		} else {
			Ok(PlaybackStatus::Playing)
		}
	}

	async fn loop_status(&self) -> fdo::Result<LoopStatus> {
		Ok(LoopStatus::None)
	}

	async fn set_loop_status(&self, _loop_status: LoopStatus) -> Result<()> {
		Ok(())
	}

	async fn rate(&self) -> fdo::Result<PlaybackRate> {
		Ok(1.0)
	}

	async fn set_rate(&self, _rate: PlaybackRate) -> Result<()> {
		Ok(())
	}

	async fn shuffle(&self) -> fdo::Result<bool> {
		Ok(false)
	}

	async fn set_shuffle(&self, _shuffle: bool) -> Result<()> {
		Ok(())
	}

	async fn volume(&self) -> fdo::Result<Volume> {
		Ok(1.)
	}

	async fn position(&self) -> fdo::Result<Time> {
		if let Some(s) = self.queue.current() && let Some(d) = s.duration {
			Ok(Time::from_secs((d as f32 * self.player.progress()) as i64))
		} else {
			Ok(Time::from_secs(0))
		}
	}

	async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
		Ok(1.)
	}

	async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
		Ok(1.)
	}

	async fn can_go_next(&self) -> fdo::Result<bool> {
		Ok(true)
	}

	async fn can_go_previous(&self) -> fdo::Result<bool> {
		Ok(true)
	}

	async fn can_play(&self) -> fdo::Result<bool> {
		Ok(true)
	}

	async fn can_pause(&self) -> fdo::Result<bool> {
		Ok(true)
	}

	async fn can_seek(&self) -> fdo::Result<bool> {
		Ok(false)
	}

	async fn can_control(&self) -> fdo::Result<bool> {
		Ok(true)
	}
}

