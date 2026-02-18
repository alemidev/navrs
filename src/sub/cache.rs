// TODO this is a nice trait flex but this cache is a bit BAD as it just grows a TON!!
//      possible solutions:
//       * auto clean songs too old and back in queue
//       * store still encoded to save space
//       * cache to disk instead of in-mem with hashes'n'stuff so we get persistent caching yeaaaa

use std::sync::OnceLock;
use super::{Song, Id};

use dashmap::DashMap;
use submarine::api::stream::StreamOptions;

#[derive(Debug, thiserror::Error)]
pub enum SongLoadError {
	#[error("subsonic api error: {0} - {0:?}")]
	Submarine(#[from] submarine::SubsonicError),

	#[error("decode error: {0} - {0:?}")]
	Symphonia(#[from] symphonia::core::errors::Error),
}

// TODO omg what happened here....
pub trait Cache<T: Clone + Sync> {
	type Error: std::error::Error + Send;
	type Fetcher: Send;

	fn contains(&self, id: &Id) -> bool;
	fn put(&self, id: Id, val: T);
	fn lookup(&self, id: &Id) -> Option<T>;
	fn fetch(&self, id: &Id, ctx: Self::Fetcher) -> impl std::future::Future<Output = Result<T, Self::Error>> + std::marker::Send;


	#[allow(unused)]
	fn load(&self, id: &Id, ctx: Self::Fetcher) -> impl std::future::Future<Output = Result<T, Self::Error>> + std::marker::Send
	where Self: Sync
	{
		async {
			if self.contains(id) && let Some(x) = self.lookup(id) {
				return Ok(x);
			}
			let x = self.fetch(id, ctx).await?;
			self.put(id.clone(), x.clone());
			Ok(x)
		}
	}

	fn prime(&self, id: &Id, ctx: Self::Fetcher) -> impl std::future::Future<Output = Result<(), Self::Error>> + std::marker::Send
	where Self: Sync
	{
		async {
			if self.contains(id) {
				return Ok(());
			}
			let x = self.fetch(id, ctx).await?;
			self.put(id.clone(), x.clone());
			Ok(())
		}
	}
}


pub fn meta() -> &'static impl Cache<Song, Error = submarine::SubsonicError, Fetcher = submarine::Client> {
	static META_CACHE: OnceLock<DashMap<Id, Song>> = OnceLock::new();
	META_CACHE.get_or_init(DashMap::default)
}

impl Cache<Song> for DashMap<Id, Song> {
	type Error = submarine::SubsonicError;
	type Fetcher = submarine::Client;

	fn contains(&self, id: &Id) -> bool {
		self.contains_key(id)
	}
	fn put(&self, id: Id, val: Song) {
		self.insert(id, val);
	}
	fn lookup(&self, id: &Id) -> Option<Song> {
		self.get(id).map(|v| v.value().clone())
	}
	async fn fetch(&self, id: &Id, ctx: submarine::Client) -> Result<Song, submarine::SubsonicError> {
		ctx.get_song(id).await
	}
}

// TODO ugly af way to pass a config here....
pub static DATA_CACHE_PATH: OnceLock<String> = OnceLock::new();


pub fn data() -> &'static impl Cache<(Vec<f32>, u32), Error = SongLoadError, Fetcher = submarine::Client> {
	static DATA_CACHE: OnceLock<DashMap<Id, (Vec<f32>, u32)>> = OnceLock::new();
	DATA_CACHE.get_or_init(DashMap::default)
}

impl Cache<(Vec<f32>, u32)> for DashMap<Id, (Vec<f32>, u32)> {
	type Error = SongLoadError;
	type Fetcher = submarine::Client;

	fn contains(&self, id: &Id) -> bool {
		self.contains_key(id)
	}
	fn put(&self, id: Id, val: (Vec<f32>, u32)) {
		self.insert(id, val);
	}
	fn lookup(&self, id: &Id) -> Option<(Vec<f32>, u32)> {
		self.get(id).map(|v| v.value().clone())
	}
	async fn fetch(&self, id: &Id, ctx: submarine::Client) -> Result<(Vec<f32>, u32), SongLoadError> {
		let mut cache_path = None;
		if let Some(cache_base) = DATA_CACHE_PATH.get() {
			cache_path = Some(format!("{cache_base}/{id}"));
		}

		if let Some(cache_path) = cache_path.as_ref() {
			match tokio::fs::try_exists(&cache_path).await {
				Err(e) => log::error!("could not check if file exists: {e}"),
				Ok(false) => {}, // not cached: must stream
				Ok(true) => match tokio::fs::read(&cache_path).await {
					Err(e) => log::error!("could not load cached file: {e}"),
					Ok(data) => {
						log::info!("song '{id}' loaded from filesystem");
						return Ok(crate::audio::decoder::decode(&data, None)?);
					},
				}
			}
		}

		log::info!("streaming song '{id}'...");
		let song = ctx.stream(
			StreamOptions {
				id: id.as_str(),
				..Default::default()
			},
			Some(std::time::Duration::from_secs(300)),
		)
			.await?;

		if let Some(cache_path) = cache_path {
			match tokio::fs::write(cache_path, &song).await {
				Err(e) => log::error!("could not save streamed song to disk: {e}"),
				Ok(()) => log::info!("saved '{id}' to disk"),
			}
		}

		Ok(crate::audio::decoder::decode(&song, None)?)
	}
}
