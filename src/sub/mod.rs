pub mod cache;
pub mod loader;
pub mod mpris;
pub mod provider;
pub mod worker;

// TODO split down this file!
pub use loader::Loader;
pub use provider::Provider;

pub type Id = std::sync::Arc<str>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Artist {
	pub id: Id,
	pub name: String,
}

impl From<submarine::data::ArtistId3> for Artist {
	fn from(value: submarine::data::ArtistId3) -> Self {
		Self {
			id: value.id.into(),
			name: value.name,
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Album {
	pub id: Id,
	pub name: String,
	pub artist_id: Option<Id>,
	pub year: Option<i32>,
}

impl From<submarine::data::Child> for Album {
	fn from(value: submarine::data::Child) -> Self {
		Self {
			id: value.id.into(),
			name: value.name,
			artist_id: value.artist_id.map(|i| i.into()),
			year: value.year,
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Song {
	pub id: Id,
	pub title: String,
	pub artist: Option<String>,
	pub artist_id: Option<Id>,
	pub album: Option<String>,
	pub album_id: Option<Id>,
	pub year: Option<i32>,
	pub track: Option<i32>,
	pub duration: Option<i32>,
	pub play_count: Option<i64>,
	pub starred: bool,
	pub content_type: Option<String>,
	pub bit_rate: Option<i32>,
}

impl From<submarine::data::Child> for Song {
	fn from(value: submarine::data::Child) -> Self {
		Self {
			id: value.id.into(),
			title: value.title,
			artist: value.artist,
			artist_id: value.artist_id.map(|i| i.into()),
			album: value.album,
			album_id: value.album_id.map(|i| i.into()),
			year: value.year,
			track: value.track,
			duration: value.duration,
			play_count: value.play_count,
			starred: value.starred.is_some(),
			content_type: value.content_type,
			bit_rate: value.bit_rate,
		}
	}
}
