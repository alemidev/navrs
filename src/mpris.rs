use mpris_server::{
	LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, RootInterface, Server, Time, TrackId, Volume, zbus::{Result, fdo}
};

use crate::music::{MusicProvider, SubsonicProvider};

pub struct SubtuiPlayer(pub SubsonicProvider);

impl RootInterface for SubtuiPlayer {
	async fn identity(&self) -> fdo::Result<String> {
		Ok("subtui".into())
	}

	async fn raise(&self) -> fdo::Result<()> { Ok(()) }
	async fn can_raise(&self) -> fdo::Result<bool> { Ok(false) }

	async fn quit(&self) -> fdo::Result<()> { Ok(()) }
	async fn can_quit(&self) -> fdo::Result<bool> { Ok(false) }

	async fn fullscreen(&self) -> fdo::Result<bool> { Ok(false) }
	async fn set_fullscreen(&self, _fullscreen:bool) -> Result<()> { Ok(()) }
	async fn can_set_fullscreen(&self) -> fdo::Result<bool> { Ok(false) }

	async fn has_track_list(&self) -> fdo::Result<bool> { Ok(false) } // TODO can do this!
	async fn desktop_entry(&self) -> fdo::Result<String> { Ok(String::new()) }

	async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> { Ok(Vec::new()) }
	async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> { Ok(Vec::new()) }
}

impl PlayerInterface for SubtuiPlayer {
	async fn set_volume(&self, _volume: Volume) -> Result<()> { Ok(()) } // TODO need volume...

	async fn metadata(&self) -> fdo::Result<Metadata> {
		let metadata = Metadata::builder()
			.title("My Song")
			.artist(["My Artist"])
			.album("My Album")
			.length(Time::from_micros(123))
			.build();
		Ok(metadata)
	}

	async fn next(&self) -> fdo::Result<()> {
		self.0.next();
		Ok(())
	}

	async fn previous(&self) -> fdo::Result<()> {
		self.0.restart();
		Ok(())
	}

	async fn pause(&self) -> fdo::Result<()> {
		self.0.pause();
		Ok(())
	}

	async fn play_pause(&self) -> fdo::Result<()> {
		log::info!("MPRIS play/pause!");
		self.0.toggle();
		Ok(())
	}

	async fn stop(&self) -> fdo::Result<()> {
		self.0.pause();
		Ok(())
	}

	async fn play(&self) -> fdo::Result<()> {
		self.0.resume();
		Ok(())
	}

	async fn seek(&self, _offset: Time) -> fdo::Result<()> {
		// TODO doable but annoying
		Ok(())
	}

	async fn set_position(&self, _track_id: TrackId, _position: Time) -> fdo::Result<()> {
		Ok(())
	}

	async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
		Ok(())
	}

	async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
		if self.0.running() {
			Ok(PlaybackStatus::Playing)
		} else {
			Ok(PlaybackStatus::Paused)
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
		Ok(Time::from_secs(0))
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

pub async fn serve(provider: crate::music::SubsonicProvider) -> Result<()> {
	log::info!("preparing MPRIS server");
	let _server = Server::new("dev.alemi.subtui", SubtuiPlayer(provider)).await?;
	let _: () = std::future::pending().await;
	Ok(())
}
