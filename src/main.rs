mod audio;
mod music;
mod app;
mod logger;
mod ui;


use clap::Parser;

use crate::audio::AudioController;

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
	logger::init().unwrap();
	let cli = Cli::parse();

	let provider = music::SubsonicProvider::connect(&cli.host, &cli.username, &cli.password).unwrap();
	let mut sink = audio::AudioSink::init(provider.clone()).unwrap();
	sink.play();

	let term = ratatui::init();
	let res = app::App::new(provider, sink).run(term);
	ratatui::restore();

	res.unwrap();
}
