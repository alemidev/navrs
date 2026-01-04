
pub trait Loader {
	type Error: std::error::Error;

	async fn all_songs(&self) -> Result<Vec<super::Song>, Self::Error>;
	async fn all_albums(&self) -> Result<Vec<super::Album>, Self::Error>;
	async fn all_artists(&self) -> Result<Vec<super::Artist>, Self::Error>;
}

impl Loader for submarine::Client {
	type Error = submarine::SubsonicError;

	async fn all_songs(&self) -> Result<Vec<super::Song>, Self::Error> {
		let mut buffer = Vec::new();
		let mut offset = 0;
		let size = 50;

		loop {
			let mut res = self.search3(
				"",
				Some(0),
				None,
				Some(0),
				None,
				Some(size as i32),
				Some(offset as i32),
				None::<String>,
			).await?;
			let n = res.song.len();
			buffer.append(&mut res.song);
			offset += n;
			if n < size {
				break;
			}
		}
		
		Ok(buffer)
	}

	async fn all_albums(&self) -> Result<Vec<super::Album>, Self::Error> {
		let mut buffer = Vec::new();
		let mut offset = 0;
		let size = 50;
		loop {
			let mut res = self.get_album_list(
				submarine::api::get_album_list::Order::Random,
				Some(size),
				Some(offset),
				None::<String>,
			).await?;
			let n = res.len();
			buffer.append(&mut res);
			offset += n;
			if n < size {
				break;
			}
		}
		
		Ok(buffer)
	}
	async fn all_artists(&self) -> Result<Vec<super::Artist>, Self::Error> {
		let mut buffer = Vec::new();
		let res = self.get_artists(None::<String>).await?;
		for mut index in res {
			buffer.append(&mut index.artist);
		}
		Ok(buffer)
	}
}
