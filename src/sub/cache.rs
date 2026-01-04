// TODO this is a nice trait flex but this cache is a bit BAD as it just grows a TON!!
//      possible solutions:
//       * auto clean songs too old and back in queue
//       * store still encoded to save space
//       * cache to disk instead of in-mem with hashes'n'stuff so we get persistent caching yeaaaa

use std::sync::OnceLock;
use super::{Song, Id};

use dashmap::DashMap;

// TODO omg what happened here....
pub trait Cache<T: Clone + Sync> {
	type Error: std::error::Error + Send;
	type Fetcher: Send;

	fn put(&self, id: Id, val: T);
	fn lookup(&self, id: &Id) -> Option<T>;
	fn fetch(&self, id: &Id, ctx: Self::Fetcher) -> impl std::future::Future<Output = Result<T, Self::Error>> + std::marker::Send;

	fn load(&self, id: &Id, ctx: Self::Fetcher) -> impl std::future::Future<Output = Result<T, Self::Error>> + std::marker::Send
	where Self: Sync
	{
		async {
			if let Some(x) = self.lookup(id) {
				return Ok(x);
			}
			let x = self.fetch(id, ctx).await?;
			self.put(id.clone(), x.clone());
			Ok(x)
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


pub fn data() -> &'static impl Cache<Vec<f32>, Error =  submarine::SubsonicError, Fetcher = submarine::Client> {
	static DATA_CACHE: OnceLock<DashMap<Id, Vec<f32>>> = OnceLock::new();
	DATA_CACHE.get_or_init(DashMap::default)
}

impl Cache<Vec<f32>> for DashMap<Id, Vec<f32>> {
	type Error = submarine::SubsonicError;
	type Fetcher = submarine::Client;

	fn put(&self, id: Id, val: Vec<f32>) {
		self.insert(id, val);
	}
	fn lookup(&self, id: &Id) -> Option<Vec<f32>> {
		self.get(id).map(|v| v.value().clone())
	}
	async fn fetch(&self, id: &Id, ctx: submarine::Client) -> Result<Vec<f32>, submarine::SubsonicError> {
		let song = ctx.stream(
			id,
			None,
			Some("subtui"),
			None,
			None::<String>,
			None,
			None
		)
			.await?;
		Ok(crate::audio::decoder::decode_mp3(&song))
	}
}
