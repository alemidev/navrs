use crate::{audio::{api::Player, decoder::SongData}, ext, sub::{self, Loader, cache::Cache}};

pub enum Op {
	RefreshLikes,
	RefreshArtists,
	RefreshAlbums,
	RefreshSongs,
	Search(String),
	Scrobble(sub::Id),
	UpdateMPRIS,
}

pub struct ProviderWorker {
	pub rx: tokio::sync::mpsc::UnboundedReceiver<Op>,
	pub player: crate::audio::sink::AudioPlayer,
	pub likes: ext::atomic::Sync<Vec<sub::Song>>,
	pub client: submarine::Client,
	pub queue: ext::atomic::Queue<sub::Song>,
	pub search: ext::atomic::Sync<Vec<sub::Song>>,
	pub cfg: crate::config::Config,
	// TODO overdoing this a bit... need a better way than 2 channels, ouchh
	pub artists: ext::atomic::Sync<Vec<sub::Artist>>,
	pub albums: ext::atomic::Sync<Vec<sub::Song>>,
	pub songs: ext::atomic::Sync<Vec<sub::Song>>,
}

impl ProviderWorker {
	pub async fn work(mut self, mpris: mpris_server::Server<sub::Provider>) {
		let mut last_fetch = std::time::SystemTime::now();

		// TODO ughh yet another bunch of copies.......
		let provider = mpris.imp().clone();
		let _client = self.client.clone();
		let _preload = self.cfg.player.preload;
		tokio::spawn(async move {
			loop {
				for i in 0.._preload {
					if let Some(song) = provider.queue.get(provider.queue.index() + i)
						&& !sub::cache::data().contains(&song.id)
					{
						provider.working.set(true);
						Self::preload(song.id, &_client, &provider.player, &provider.queue).await;
						provider.working.set(false);
						provider.update_mpris();
						break;
					}
				}

				tokio::time::sleep(std::time::Duration::from_secs(1)).await;
			}
		});

		while let Some(op) = self.rx.recv().await {
			match op {
				Op::UpdateMPRIS => self.update_mpris(&mpris).await,
				Op::RefreshLikes => {
					self.reload_likes().await;
					last_fetch = std::time::SystemTime::now();
				},
				Op::Search(query) => {
					log::info!("searching '{query}'");
					match self.client.search3(
						query,
						Some(50),
						None,
						Some(50),
						None,
						Some(50),
						None,
						None::<String>,
					)
						.await
					{
						Err(e) => log::error!("error searching: {e}"),
						Ok(x) => self.search.set(x.song),
					}
				},
				Op::RefreshArtists => {
					log::info!("refreshing artists...");
					match self.client.all_artists().await {
						Err(e) => log::error!("error loading all artists: {e}"),
						Ok(artists) => self.artists.set(artists),
					}
				},
				Op::RefreshSongs => {
					log::info!("refreshing songs...");
					match self.client.all_songs().await {
						Err(e) => log::error!("error loading all songs: {e}"),
						Ok(songs) => self.songs.set(songs),
					}
				},
				Op::RefreshAlbums => {
					log::info!("refreshing albums...");
					match self.client.all_albums().await {
						Err(e) => log::error!("error loading all albums: {e}"),
						Ok(albums) => self.albums.set(albums),
					}
				},
				Op::Scrobble(id) => {
					log::info!("scrobbling play of '{id}'");
					if let Err(e) = self.client.scrobble(vec![(id, None)], Some(true)).await {
						log::error!("error scrobbling song: {e}");
					}
				},
			}

			if std::time::SystemTime::now() > last_fetch + std::time::Duration::from_secs(300) {
				self.reload_likes().await;
				last_fetch = std::time::SystemTime::now();
			}

		}
		log::info!("provider worker quitting");
	}

	async fn preload(id: sub::Id, client: &submarine::Client, sink: &crate::audio::sink::AudioPlayer, queue: &ext::atomic::Queue<sub::Song>) {
		match sub::cache::data().prime(&id, client.clone()).await {
			Err(e) => log::error!("error preloading data for song '{id}': {e}"),
			Ok(()) => {
				if let Some(s) = queue.current() && s.id == id && sink.is_empty() {
					let SongData { data, sample_rate } = sub::cache::data().lookup(&id).expect("just primed");
					log::info!("setting playback buffer");
					if let Err(e) = sink.play(data, sample_rate) {
						log::error!("error playing song: {e}");
					}
				}
			},
		}
		if let Err(e) = sub::cache::meta().prime(&id, client.clone()).await {
			log::error!("error preloading meta for song '{id}': {e}")
		}
	}

	async fn reload_likes(&self) {
		log::info!("reloading likes");
		match self.client.get_starred(None::<String>).await {
			Ok(data) => self.likes.set(data.song),
			Err(e) => log::error!("error fetching likes: {e}"),
		}
	}

	async fn update_mpris(&self, mpris: &mpris_server::Server<sub::Provider>) {
		let mut properties = Vec::new();
		
		if self.player.paused() {
			properties.push(mpris_server::Property::PlaybackStatus(mpris_server::PlaybackStatus::Paused));
		} else {
			properties.push(mpris_server::Property::PlaybackStatus(mpris_server::PlaybackStatus::Playing));
		}

		if let Some(song) = self.queue.current() {
			properties.push(mpris_server::Property::Metadata(
				mpris_server::Metadata::builder()
					.title(song.title)
					.artist(song.artist.map(|x| vec![x]).unwrap_or_default())
					.album(song.album.unwrap_or_default())
					.length(mpris_server::Time::from_secs(song.duration.unwrap_or_default() as i64))
					.build()
			));
		}

		if let Err(e) = mpris.properties_changed(properties).await {
			log::error!("error updating MPRIS metadata: {e} - {e:?}");
		}
	}
}
