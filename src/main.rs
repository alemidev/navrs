mod audio;
mod music;


use clap::Parser;

use crate::{audio::AudioController, music::MusicProvider};

/// A TUI Subsonic music player
#[derive(Parser)]
struct Cli {
	/// username to use for authentication against server
	#[arg(short, long)]
	username: String,

	/// password to use for authentication against server
	#[arg(short, long)]
	password: String,

	/// address of your server
	#[arg(short = 'H', long)]
	host: String,
}

fn main() {
	env_logger::init();
	let cli = Cli::parse();

	let mut provider = music::SubsonicProvider::connect(&cli.host, &cli.username, &cli.password).unwrap();
	let mut sink = audio::AudioSink::init(provider.clone()).unwrap();
	sink.play();

	loop {
		if let Some(s) = provider.random_song().unwrap() {
			println!(">> {s}");
		} else {
			println!("no song?");
		}
		provider.wait();
	}
}
